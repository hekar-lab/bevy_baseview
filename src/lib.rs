//#![allow(unused_imports, dead_code, unused_mut, unused_variables)]
mod conversions;
mod default_plugins;
mod keyboard;
mod parent_window;

use bevy::input::ButtonState;
use bevy::ecs::event::Event;
use bevy::ecs::system::{Resource, SystemState};
use bevy::log::info;
use bevy::prelude::{Commands, Entity, EventReader, EventWriter, FromWorld, Query, Res, With};
pub use parent_window::ParentWindow;
use rwh_05::{HasRawDisplayHandle, HasRawWindowHandle};

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use bevy::app::{App, Plugin, PluginsState};
use bevy::input::{
    keyboard::KeyboardInput,
    mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel},
};
use bevy::math::DVec2;
use bevy::window::{
    CursorEntered, CursorLeft, CursorMoved, PrimaryWindow, RawHandleWrapper, RequestRedraw, Window, WindowBackendScaleFactorChanged, WindowCreated, WindowFocused, WindowResized, WindowScaleFactorChanged
};
use lazy_static::lazy_static;

//use baseview_windows::BaseviewWindows;
pub use default_plugins::DefaultBaseviewPlugins;

/// Container for user-provided information about the parent window.
#[derive(Resource)]
struct BaseviewWindowInfo {
    parent_win: ParentWindow,
    window_open_options: baseview::WindowOpenOptions,
}

impl std::fmt::Debug for BaseviewWindowInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Note, we do not support glContext for baseview.
        let window_open_options_str = format!(
            "WindowOpenOptions(title={:?}, size={:?}, scale={:?})",
            self.window_open_options.title,
            self.window_open_options.size,
            self.window_open_options.scale
        );
        f.debug_struct("BaseviewWindowInfo")
            .field("parent_win", &self.parent_win)
            .field("window_open_options", &window_open_options_str)
            .finish()
    }
}

impl Clone for BaseviewWindowInfo {
    fn clone(&self) -> Self {
        Self {
            parent_win: self.parent_win.clone(),
            window_open_options: clone_window_options(&self.window_open_options),
        }
    }
}

unsafe impl Sync for BaseviewWindowInfo {}
unsafe impl Send for BaseviewWindowInfo {}

/**
 * An AppProxy helps callers control the lifetime of the bevy app,
 * since it's stored statically and run within the window event loop
 * (which is triggered by the host). Dropping the AppProxy will drop
 * the bevy App.
 */
pub struct AppProxy;

impl Drop for AppProxy {
    fn drop(&mut self) {
        // TODO: GUI Thread is definitely not guaranteed; need to evaluate.
        drop_app(&GuiThread);
    }
}

/**
 * Attach a baseview window to an app.
 */
pub fn attach_to<P: Into<ParentWindow>>(
    app: &mut App,
    window_open_options: &baseview::WindowOpenOptions,
    parent: P,
) -> AppProxy {
    let parent_win = parent.into();
    let window_open_options = clone_window_options(window_open_options);

    let baseview_window_info = BaseviewWindowInfo {
        parent_win,
        window_open_options,
    };
    app.insert_resource(baseview_window_info);

    AppProxy
}

// h/t to iced_baseview
fn clone_window_options(window: &baseview::WindowOpenOptions) -> baseview::WindowOpenOptions {
    baseview::WindowOpenOptions {
        title: window.title.clone(),
        #[cfg(feature = "baseviewgl")]
        gl_config: window
            .gl_config
            .as_ref()
            .map(|config| baseview::gl::GlConfig { ..*config }),
        ..*window
    }
}

#[derive(Default)]
pub struct BaseviewPlugin;

// Marker thread for indicating that something should only be called from the GUI thread. Not
// enforced, purely for internal documentation.
struct GuiThread;

impl Plugin for BaseviewPlugin {
    fn name(&self) -> &str {
        "bevy_baseview::BaseviewPlugin"
    }

    fn build(&self, app: &mut App) {
        app//.init_non_send_resource::<BaseviewWindows>()
            .set_runner(baseview_runner)
            .add_event::<CloseAppRequest>();
    }
}

#[derive(Clone, Debug, Default, Event)]
pub struct CloseAppRequest;

#[derive(Clone, Debug, Default, Event)]
pub struct CloseAppResponse;

#[derive(Clone, Copy)]
pub struct Update;

fn baseview_runner(mut app: App) {
    // TODO find a cleaner way to wait for the PluginsState to be ready :p
    loop {
        if app.plugins_state() == PluginsState::Ready {
            app.finish();
            app.cleanup();
            break;
        }
        // Avoid spam
        std::thread::sleep(Duration::from_millis(50));
    }

    let mut runner_system_state: SystemState<Res<BaseviewWindowInfo>> = SystemState::from_world(&mut app.world);
    let baseview_info = runner_system_state.get_mut(&mut app.world);

    let BaseviewWindowInfo {
        window_open_options,
        parent_win,
    } = baseview_info.clone();

    let (
        send_update, 
        recv_update
    ) = crossbeam_channel::bounded::<Update>(1);

    let arc_app = Arc::new(Mutex::new(app));
    let baseview_window = BaseviewWindow::new(arc_app.clone(), send_update);
    let win_app = arc_app.clone();
    let _handle = baseview::Window::open_parented(
        &parent_win, 
        window_open_options, 
        move |base_window| {
            let mut app = win_app.lock().unwrap();
            
            let mut create_window_system_state: SystemState<(
                Commands,
                Query<(Entity, &mut Window), With<PrimaryWindow>>,
                EventWriter<WindowCreated>,
            )> = SystemState::from_world(&mut app.world);

            let (
                mut commands,
                mut windows,
                mut event_writer,
            ) = create_window_system_state.get_mut(&mut app.world);

            let (entity, window) = windows.single_mut();

            info!(
                "Creating new window {:?} ({:?})",
                window.title.as_str(),
                entity
            );

            commands
                .entity(entity)
                .insert(RawHandleWrapper{
                    window_handle: base_window.raw_window_handle(),
                    display_handle: base_window.raw_display_handle(),
                });

            event_writer.send(WindowCreated { window: entity });

            create_window_system_state.apply(&mut app.world);

            baseview_window
        }
    );

    loop {
        match recv_update.recv() {
            Ok(_) => {
                match arc_app.lock() {
                    Ok(mut app) => {
                        app.update()
                    },
                    _ => {}
                }
            },

            Err(e) => {
                //log::warn!("Error during update recv routine: {:?}", e);
            }
        }
    }
}

fn drop_app(_gui: &GuiThread) {
    log::trace!("Dropping App");
}

lazy_static! {
    static ref BASEVIEW_WINDOW_ID: AtomicU64 = AtomicU64::new(0);
}

#[derive(Debug)]
pub struct BaseviewWindow {
    id: u64,
    app: Arc<Mutex<App>>,
    send_update: crossbeam_channel::Sender<Update>,
    last_scale_factor: f64,
    pending_events: VecDeque<baseview::Event>,
}

struct EventStatus {
    return_status: baseview::EventStatus,
    shutdown: bool,
}

impl BaseviewWindow {
    pub fn new(app: Arc<Mutex<App>>, send_update: crossbeam_channel::Sender<Update>) -> Self {
        let id = BASEVIEW_WINDOW_ID.fetch_add(1, Ordering::AcqRel);
        Self {
            id,
            app,
            send_update,
            last_scale_factor: 1.0,
            pending_events: VecDeque::new(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    fn process_pending_events(&mut self, app_lock: &mut MutexGuard<App>) -> EventStatus {
        let mut status = EventStatus {
            return_status: baseview::EventStatus::Captured,
            shutdown: false,
        };

        while !self.pending_events.is_empty() {
            let pending_event = self.pending_events.pop_front().unwrap();
            let pending_status = self.process_event(pending_event, app_lock);
            if pending_status.shutdown {
                status.shutdown = true;
            }
        }

        status
    }

    fn process_event(&mut self, event: baseview::Event, app: &mut MutexGuard<App>) -> EventStatus {
        let mut status = EventStatus {
            return_status: baseview::EventStatus::Captured,
            shutdown: false,
        };

        let mut process_event_system_state: SystemState<(
            EventReader<CloseAppRequest>,
            //EventWriter<CloseAppResponse>,

            EventWriter<CursorMoved>,
            EventWriter<CursorEntered>,
            EventWriter<CursorLeft>,

            EventWriter<MouseButtonInput>,
            EventWriter<MouseWheel>,

            EventWriter<KeyboardInput>,

            EventWriter<WindowFocused>,
            EventWriter<WindowResized>,
            EventWriter<WindowScaleFactorChanged>,
            EventWriter<WindowBackendScaleFactorChanged>,

            Query<(Entity, &mut Window), With<PrimaryWindow>>,
        )> = SystemState::from_world(&mut app.world);

        let (
            mut close_app_requests,
            //mut close_app_responses,

            mut cursor_moved_events,
            mut cursor_entered_events,
            mut cursor_left_events,

            mut mouse_button_input_events,
            mut mouse_wheel_events,

            mut keyboard_input_events,

            mut window_focused_events,
            mut window_resized_events,
            mut window_scale_factor_changed_evnets,
            mut window_backend_scale_factor_changed_events,

            mut window_entity
        ) = process_event_system_state.get_mut(&mut app.world);

        for _ in close_app_requests.read() {
            status.shutdown = true;
            //close_app_responses.send(CloseAppResponse);
        }

        match event {
            baseview::Event::Mouse(e) => {
                match e {
                    baseview::MouseEvent::CursorMoved { position, .. } => {
                        match window_entity.get_single_mut(){
                            Ok((entity, mut window)) => {
                                let position = DVec2::new(position.x, position.y);
                                window.set_physical_cursor_position(Some(position));
                                cursor_moved_events.send(CursorMoved {
                                    window: entity,
                                    position: position.as_vec2(),
                                });
                            },
                            Err(err) => {
                                log::info!("Skipped event for closed window: {:?}", err);
                                return status;
                            },
                        }
                    }
                    baseview::MouseEvent::CursorEntered => {
                        match window_entity.get_single_mut(){
                            Ok((entity, _window)) => {
                                cursor_entered_events.send(CursorEntered { window: entity });
                            },
                            Err(err) => {
                                log::info!("Skipped event for closed window: {:?}", err);
                                return status;
                            },
                        }
                    }
                    baseview::MouseEvent::ButtonPressed { button, .. } => {
                        match window_entity.get_single_mut(){
                            Ok((entity, _window)) => {
                                mouse_button_input_events.send(MouseButtonInput {
                                    window: entity,
                                    button: conversions::baseview_mousebutton_to_bevy(button),
                                    state: ButtonState::Pressed,
                                });
                            },
                            Err(err) => {
                                log::info!("Skipped event for closed window: {:?}", err);
                                return status;
                            },
                        }
                    }
                    baseview::MouseEvent::ButtonReleased { button, .. } => {
                        match window_entity.get_single_mut(){
                            Ok((entity, _window)) => {
                                mouse_button_input_events.send(MouseButtonInput {
                                    window: entity,
                                    button: conversions::baseview_mousebutton_to_bevy(button),
                                    state: ButtonState::Released,
                                });
                            },
                            Err(err) => {
                                log::info!("Skipped event for closed window: {:?}", err);
                                return status;
                            },
                        }
                    }
                    baseview::MouseEvent::CursorLeft => {
                        match window_entity.get_single_mut(){
                            Ok((entity, _window)) => {
                                cursor_left_events.send(CursorLeft { window: entity });
                            },
                            Err(err) => {
                                log::info!("Skipped event for closed window: {:?}", err);
                                return status;
                            },
                        }
                    }
                    baseview::MouseEvent::WheelScrolled { delta, .. } => match delta {
                        baseview::ScrollDelta::Lines { x, y } => {
                            match window_entity.get_single_mut(){
                                Ok((entity, _window)) => {
                                    mouse_wheel_events.send(MouseWheel {
                                        window: entity,
                                        unit: MouseScrollUnit::Line,
                                        x,
                                        y,
                                    });
                                },
                                Err(err) => {
                                    log::info!("Skipped event for closed window: {:?}", err);
                                    return status;
                                },
                            }
                        }
                        baseview::ScrollDelta::Pixels { x, y } => {
                            match window_entity.get_single_mut(){
                                Ok((entity, _window)) => {
                                    mouse_wheel_events.send(MouseWheel {
                                        window: entity,
                                        unit: MouseScrollUnit::Pixel,
                                        x,
                                        y,
                                    });
                                },
                                Err(err) => {
                                    log::info!("Skipped event for closed window: {:?}", err);
                                    return status;
                                },
                            }
                        }
                    },
                    _ => {}
                };
            }
            baseview::Event::Keyboard(e) => {
                match window_entity.get_single_mut(){
                    Ok((entity, _window)) => {
                        let key_code = keyboard::key_to_keycode(e.key);
                        let state = match e.state {
                            keyboard_types::KeyState::Down => ButtonState::Pressed,
                            keyboard_types::KeyState::Up => ButtonState::Released,
                        };
                        let event = KeyboardInput {
                            window: entity,
                            scan_code: keyboard::code_to_scancode(e.code),
                            key_code,
                            state,
                        };
                        keyboard_input_events.send(event);
                    },
                    Err(err) => {
                        log::info!("Skipped event for closed window: {:?}", err);
                        return status;
                    },
                }
            }
            baseview::Event::Window(window_event) => {
                match window_entity.get_single_mut(){
                    Ok((entity, mut window)) => {
                        match window_event {
                            baseview::WindowEvent::Resized(window_info) => {
                                // First adjust scale, if needed.
                                let scale_factor = window_info.scale();
        
                                if scale_factor != self.last_scale_factor {
                                    window_backend_scale_factor_changed_events.send(
                                        WindowBackendScaleFactorChanged {
                                            window: entity,
                                            scale_factor,
                                        },
                                    );
        
                                    window_scale_factor_changed_evnets.send(WindowScaleFactorChanged {
                                        window: entity,
                                        scale_factor,
                                    });
        
                                    self.last_scale_factor = window_info.scale();
                                    window.resolution.set_scale_factor(self.last_scale_factor);
                                }
        
                                window.resolution.set_physical_resolution(
                                    window_info.physical_size().width,
                                    window_info.physical_size().height,
                                );
                                window_resized_events.send(WindowResized {
                                    window: entity,
                                    width: window_info.logical_size().width as f32,
                                    height: window_info.logical_size().height as f32,
                                });
                            }
                            baseview::WindowEvent::Focused => {
                                window.focused = true;
                                window_focused_events.send(WindowFocused {
                                    window: entity,
                                    focused: true,
                                });
                            }
                            baseview::WindowEvent::Unfocused => {
                                window.focused = false;
                                window_focused_events.send(WindowFocused {
                                    window: entity,
                                    focused: false,
                                });
                            }
                            baseview::WindowEvent::WillClose => {
                                status.shutdown = true;
                            }
                        }
                    },
                    Err(err) => {
                        log::info!("Skipped event for closed window: {:?}", err);
                        return status;
                    },
                }
            }
        }
        status
    }
}

impl Drop for BaseviewWindow {
    fn drop(&mut self) {
        log::info!("BaseviewWindow: drop");
    }
}

impl baseview::WindowHandler for BaseviewWindow {
    fn on_frame(&mut self, _window: &mut baseview::Window) {
        if let Ok(mut lock) = self.app.clone().lock() {
            self.process_pending_events(&mut lock);

            let mut frame_system_state: SystemState<(
                EventWriter<RequestRedraw>,

                Query<(Entity, &mut Window), With<PrimaryWindow>>,
            )> = SystemState::from_world(&mut lock.world);

            let (
                mut request_redraw_events,

                mut window_entity,
            ) = frame_system_state.get_mut(&mut lock.world);

            request_redraw_events.send(RequestRedraw);

            match window_entity.get_single_mut() {
                Ok((_entity, window)) => {
                    if window.focused {
                        //lock.update();
                        if let Err(e) = self.send_update.send(Update) {
                            //log::warn!("Error during update send routine: {:?}", e);
                        }
                    }
                }, 
                _ => {}
            }
        }
    }

    fn on_event(
        &mut self,
        _window: &mut baseview::Window,
        event: baseview::Event,
    ) -> baseview::EventStatus {
        let gui_thread = GuiThread;

        self.pending_events.push_back(event);

        let mut status = EventStatus {
            return_status: baseview::EventStatus::Ignored,
            shutdown: false,
        };

        if let Ok(mut lock) = self.app.clone().try_lock() {
            status = self.process_pending_events(&mut lock);
        }

        if status.shutdown {
            drop_app(&gui_thread);
        }

        status.return_status
    }
}