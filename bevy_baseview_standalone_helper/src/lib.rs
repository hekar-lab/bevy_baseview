//! App harness allowing users to create standalone bevy apps to develop the GUI independent of the
//! baseview layer.

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use bevy::window::RawHandleWrapper;
use raw_window_handle::HasRawWindowHandle;
use raw_window_handle::HasRawDisplayHandle;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use bevy_baseview_plugin::{AppProxy, ParentWindow};

// Window size (logical).
const WINDOW_WIDTH: f64 = 500.0;
const WINDOW_HEIGHT: f64 = 400.0;

struct AppWrapper<F: Fn(ParentWindow, f64, f64) -> AppProxy> {
    initialized: AtomicBool,
    parent_win: ParentWindow,
    create_app: F,
    app: Option<AppProxy>,
}

impl<F: Fn(ParentWindow, f64, f64) -> AppProxy> AppWrapper<F> {
    pub fn new(raw_handle: RawHandleWrapper, create_app: F) -> Self {
        let parent_win = ParentWindow::from(raw_handle);
        let initialized = AtomicBool::new(false);
        Self {
            initialized,
            parent_win,
            create_app,
            app: None,
        }
    }
    pub fn receive_events<'a>(
        &mut self,
        window: &winit::window::Window,
        event: Event<'a, ()>,
        control_flow: &mut ControlFlow,
    ) {
        *control_flow = ControlFlow::Poll;

        if let Ok(_) =
            self.initialized
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        {
            self.app = Some((self.create_app)(
                self.parent_win.clone(),
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            ));
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    }
}

pub fn run_app<F: Fn(ParentWindow, f64, f64) -> AppProxy + 'static>(create_app: F) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)
        .unwrap();

    let raw_handle = RawHandleWrapper {
        window_handle: window.raw_window_handle(),
        display_handle: window.raw_display_handle()
    };

    let mut app_wrapper = AppWrapper::new(raw_handle, create_app);
    event_loop.run(move |event, _, control_flow| {
        app_wrapper.receive_events(&window, event, control_flow)
    });
}
