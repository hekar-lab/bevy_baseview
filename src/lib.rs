mod window;
mod conversions;
mod keyboard;
mod parent_window;
mod default_plugins;

use std::time::Duration;

use bevy::ecs::system::SystemState;
use bevy::log::info;
use bevy::prelude::{Commands, Entity, EventWriter, FromWorld, Query, With};

use bevy::app::{App, PluginsState};
use bevy::window::{PrimaryWindow, RawHandleWrapper, Window, WindowCreated};

use rwh_05::{HasRawDisplayHandle, HasRawWindowHandle};
use window::BevyWindow;

pub use parent_window::ParentWindow;
pub use default_plugins::DefaultBaseviewPlugins;

pub fn open_parented<B>(
    parent_window: ParentWindow,
    window_open_options: baseview::WindowOpenOptions,
    app_builder: B
) 
    where
    B: FnOnce(&mut App) -> &mut App + Send + 'static
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
            )> = SystemState::from_world(&mut app.world);

            let (
                mut commands,
                mut windows,
                mut event_writer,
            ) = create_window_system_state.get_mut(&mut app.world);

            let (entity, window_comp) = windows.single_mut();

            info!(
                "Creating new window {:?} ({:?})",
                window_comp.title.as_str(),
                entity
            );

            commands
                .entity(entity)
                .insert(RawHandleWrapper{
                    window_handle: window.raw_window_handle(),
                    display_handle: window.raw_display_handle(),
                });

            event_writer.send(WindowCreated { window: entity });

            create_window_system_state.apply(&mut app.world);

            BevyWindow::new(app)
        }
    );
}