use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

use bevy::input::ButtonState;
use bevy::ecs::system::SystemState;
use bevy::prelude::{Entity, EventWriter, FromWorld, Query, With};

use bevy::app::App;
use bevy::input::{
    keyboard::KeyboardInput,
    mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel},
};
use bevy::math::DVec2;
use bevy::window::{
    CursorEntered, CursorLeft, CursorMoved, PrimaryWindow, RequestRedraw, Window, WindowBackendScaleFactorChanged, WindowFocused, WindowResized, WindowScaleFactorChanged
};

use lazy_static::lazy_static;

use crate::conversions;
use crate::keyboard;

lazy_static! {
    static ref BEVY_WINDOW_ID: AtomicU64 = AtomicU64::new(0);
}

#[derive(Debug)]
pub struct BevyWindow {
    id: u64,
    app: App,
    last_scale_factor: f64,
    pending_events: VecDeque<baseview::Event>,
}

struct EventStatus {
    return_status: baseview::EventStatus,
    shutdown: bool,
}

impl BevyWindow {
    pub fn new(app: App) -> Self {
        let id = BEVY_WINDOW_ID.fetch_add(1, Ordering::AcqRel);
        Self {
            id,
            app,
            last_scale_factor: 1.0,
            pending_events: VecDeque::new(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    fn process_pending_events(&mut self) -> EventStatus {
        let mut status = EventStatus {
            return_status: baseview::EventStatus::Captured,
            shutdown: false,
        };

        while !self.pending_events.is_empty() {
            let pending_event = self.pending_events.pop_front().unwrap();
            let pending_status = self.process_event(pending_event);
            if pending_status.shutdown {
                status.shutdown = true;
            }
        }

        status
    }

    fn process_event(&mut self, event: baseview::Event) -> EventStatus {
        let mut status = EventStatus {
            return_status: baseview::EventStatus::Captured,
            shutdown: false,
        };

        let mut process_event_system_state: SystemState<(
            //EventReader<CloseAppRequest>,
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
        )> = SystemState::from_world(&mut self.app.world);

        let (
            //mut close_app_requests,
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
        ) = process_event_system_state.get_mut(&mut self.app.world);

        // for _ in close_app_requests.read() {
        //     status.shutdown = true;
        //     //close_app_responses.send(CloseAppResponse);
        // }

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
                                    delta: None,
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
                        let key_code = keyboard::key_to_keycode(e.key.clone());
                        let state = match e.state {
                            keyboard_types::KeyState::Down => ButtonState::Pressed,
                            keyboard_types::KeyState::Up => ButtonState::Released,
                        };
                        let event = KeyboardInput {
                            window: entity,
                            key_code,
                            logical_key: keyboard::key_to_bevy_key(e.key),
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
                                    window.resolution.set_scale_factor(self.last_scale_factor as f32);
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

impl Drop for BevyWindow {
    fn drop(&mut self) {
        log::info!("BaseviewWindow: drop");
    }
}

impl baseview::WindowHandler for BevyWindow {
    fn on_frame(&mut self, _window: &mut baseview::Window) {
        self.process_pending_events();

        let mut frame_system_state: SystemState<(
            EventWriter<RequestRedraw>,

            Query<(Entity, &mut Window), With<PrimaryWindow>>,
        )> = SystemState::from_world(&mut self.app.world);

        let (
            mut request_redraw_events,

            mut window_entity,
        ) = frame_system_state.get_mut(&mut self.app.world);

        request_redraw_events.send(RequestRedraw);

        match window_entity.get_single_mut() {
            Ok((_entity, window)) => {
                if window.focused {
                    self.app.update();
                }
            }, 
            _ => {}
        }
    }

    fn on_event(
        &mut self,
        _window: &mut baseview::Window,
        event: baseview::Event,
    ) -> baseview::EventStatus {
        //let gui_thread = GuiThread;

        self.pending_events.push_back(event);

        let status = self.process_pending_events();

        // if status.shutdown {
        //     drop_app(&gui_thread);
        // }

        status.return_status
    }
}