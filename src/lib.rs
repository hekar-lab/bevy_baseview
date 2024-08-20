mod window;
mod conversions;
mod keyboard;
mod parent_window;
mod default_plugins;

use std::sync::{Arc, Mutex};

use bevy::ecs::system::SystemState;
use bevy::log::info;
use bevy::prelude::{Commands, Entity, EventWriter, FromWorld, Query, With};

use bevy::app::{App, PluginsState};
use bevy::window::{PrimaryWindow, RawHandleWrapper, RawHandleWrapperHolder, Window, WindowCreated, WindowWrapper};

use parent_window::RawWindow;
use rwh_05::HasRawWindowHandle;
use window::BevyWindow;

pub use default_plugins::DefaultBaseviewPlugins;

pub fn open_parented<P, B>(
    parent_window: P,
    window_open_options: baseview::WindowOpenOptions,
    app_builder: B
) -> baseview::WindowHandle
    where
    P: HasRawWindowHandle,
    B: FnOnce(&mut App) -> &mut App + Send + Sync + 'static
{
    baseview::Window::open_parented(
        &parent_window, 
        window_open_options, 
        |window| {
            let mut app = App::new();
            app_builder(&mut app);

            while app.plugins_state() == PluginsState::Adding {
                bevy::tasks::tick_global_task_pools_on_main_thread();
            }
            app.finish();
            app.cleanup();
        
            app.update();

            let mut create_window_system_state: SystemState<(
                Commands,
                Query<(Entity, &mut Window), With<PrimaryWindow>>,
                EventWriter<WindowCreated>,
            )> = SystemState::from_world(app.world_mut());

            let (
                mut commands,
                mut windows,
                mut event_writer,
            ) = create_window_system_state.get_mut(app.world_mut());

            let (entity, window_comp) = windows.single_mut();

            info!(
                "Creating new window {:?} ({:?})",
                window_comp.title.as_str(),
                entity
            );

            let window_wrapper = WindowWrapper::new(RawWindow::new(window));

            if let Ok(handle_wrapper) = RawHandleWrapper::new(&window_wrapper) {
                commands
                    .entity(entity)
                    .insert(handle_wrapper.clone())
                    .insert(RawHandleWrapperHolder(Arc::new(Mutex::new(Some(handle_wrapper.clone())))));

                event_writer.send(WindowCreated { window: entity });

                create_window_system_state.apply(app.world_mut());
            }

            BevyWindow::new(app)
        }
    )
}