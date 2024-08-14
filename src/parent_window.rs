use bevy::window::RawHandleWrapper;
use rwh_05::{HasRawDisplayHandle, HasRawWindowHandle, RawWindowHandle};

#[derive(Clone, Debug)]
pub struct ParentWindow(RawHandleWrapper);

unsafe impl HasRawWindowHandle for ParentWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0.window_handle
    }
}

unsafe impl HasRawDisplayHandle for ParentWindow {
    fn raw_display_handle(&self) -> rwh_05::RawDisplayHandle {
        self.0.display_handle
    }
}

impl From<RawHandleWrapper> for ParentWindow {
    fn from(inst: RawHandleWrapper) -> Self {
        ParentWindow(inst)
    }
}

impl Into<RawHandleWrapper> for ParentWindow {
    fn into(self) -> RawHandleWrapper {
        self.0
    }
}