use std::{num::NonZero, ptr::NonNull};

use bevy::window::RawHandleWrapper;

#[derive(Clone, Debug)]
pub struct ParentWindow(RawHandleWrapper);

unsafe impl rwh_06::HasRawWindowHandle for ParentWindow {
    fn raw_window_handle(&self) -> Result<rwh_06::RawWindowHandle, rwh_06::HandleError> {
        Ok(self.0.window_handle)
    }
}

unsafe impl rwh_06::HasRawDisplayHandle for ParentWindow {
    fn raw_display_handle(&self) -> Result<rwh_06::RawDisplayHandle, rwh_06::HandleError>  {
        Ok(self.0.display_handle)
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

pub struct OldRawWindowHandle(pub rwh_05::RawWindowHandle);
pub struct OldRawDisplayHandle(pub rwh_05::RawDisplayHandle);

impl Into<rwh_06::RawWindowHandle> for OldRawWindowHandle {
    fn into(self) -> rwh_06::RawWindowHandle {
        match self.0 {
            rwh_05::RawWindowHandle::UiKit(handle) => {
                rwh_06::RawWindowHandle::UiKit(
                    rwh_06::UiKitWindowHandle::new(unsafe { 
                        NonNull::new_unchecked(handle.ui_view) 
                    })
                )
            },
            rwh_05::RawWindowHandle::AppKit(handle) => {
                rwh_06::RawWindowHandle::AppKit(
                    rwh_06::AppKitWindowHandle::new(unsafe {
                        NonNull::new_unchecked(handle.ns_view)
                    })
                )
            },
            rwh_05::RawWindowHandle::Orbital(handle) => {
                rwh_06::RawWindowHandle::Orbital(
                    rwh_06::OrbitalWindowHandle::new(unsafe {
                        NonNull::new_unchecked(handle.window)
                    })
                )
            },
            rwh_05::RawWindowHandle::Xlib(handle) => {
                rwh_06::RawWindowHandle::Xlib(
                    rwh_06::XlibWindowHandle::new(handle.window)
                )
            },
            rwh_05::RawWindowHandle::Xcb(handle) => {
                rwh_06::RawWindowHandle::Xcb(
                    rwh_06::XcbWindowHandle::new(unsafe {
                        NonZero::new_unchecked(handle.window)
                    })
                )
            },
            rwh_05::RawWindowHandle::Wayland(handle) => {
                rwh_06::RawWindowHandle::Wayland(
                    rwh_06::WaylandWindowHandle::new(unsafe {
                        NonNull::new_unchecked(handle.surface)
                    })
                )
            },
            rwh_05::RawWindowHandle::Drm(handle) => {
                rwh_06::RawWindowHandle::Drm(
                    rwh_06::DrmWindowHandle::new(handle.plane)
                )
            },
            rwh_05::RawWindowHandle::Gbm(handle) => {
                rwh_06::RawWindowHandle::Gbm(
                    rwh_06::GbmWindowHandle::new(unsafe {
                        NonNull::new_unchecked(handle.gbm_surface)
                    })
                )
            },
            rwh_05::RawWindowHandle::Win32(handle) => {
                rwh_06::RawWindowHandle::Win32(
                    rwh_06::Win32WindowHandle::new(unsafe {
                        NonZero::new_unchecked(handle.hwnd as isize)
                    })
                )
            },
            rwh_05::RawWindowHandle::WinRt(handle) => {
                rwh_06::RawWindowHandle::WinRt(
                    rwh_06::WinRtWindowHandle::new(unsafe {
                        NonNull::new_unchecked(handle.core_window)
                    })
                )
            },
            rwh_05::RawWindowHandle::Web(handle) => {
                rwh_06::RawWindowHandle::Web(
                    rwh_06::WebWindowHandle::new(handle.id)
                )
            },
            rwh_05::RawWindowHandle::AndroidNdk(handle) => {
                rwh_06::RawWindowHandle::AndroidNdk(
                    rwh_06::AndroidNdkWindowHandle::new(unsafe {
                        NonNull::new_unchecked(handle.a_native_window)
                    })
                )
            },
            rwh_05::RawWindowHandle::Haiku(handle) => {
                rwh_06::RawWindowHandle::Haiku(
                    rwh_06::HaikuWindowHandle::new(unsafe {
                        NonNull::new_unchecked(handle.b_window)
                    })
                )
            },
            _ => panic!("Raw window handle conversion not supported"),
        }
    }
}

impl Into<rwh_06::RawDisplayHandle> for OldRawDisplayHandle {
    fn into(self) -> rwh_06::RawDisplayHandle {
        match self.0 {
            rwh_05::RawDisplayHandle::UiKit(_) => {
                rwh_06::RawDisplayHandle::UiKit(
                    rwh_06::UiKitDisplayHandle::new()
                )
            },
            rwh_05::RawDisplayHandle::AppKit(_) => {
                rwh_06::RawDisplayHandle::AppKit(
                    rwh_06::AppKitDisplayHandle::new()
                )
            },
            rwh_05::RawDisplayHandle::Orbital(_) => {
                rwh_06::RawDisplayHandle::Orbital(
                    rwh_06::OrbitalDisplayHandle::new()
                )
            },
            rwh_05::RawDisplayHandle::Xlib(handle) => {
                rwh_06::RawDisplayHandle::Xlib(
                    rwh_06::XlibDisplayHandle::new(
                        Some(unsafe {
                            NonNull::new_unchecked(handle.display)
                        }),
                        handle.screen
                    )
                )
            },
            rwh_05::RawDisplayHandle::Xcb(handle) => {
                rwh_06::RawDisplayHandle::Xcb(
                    rwh_06::XcbDisplayHandle::new(
                        Some(unsafe {
                            NonNull::new_unchecked(handle.connection)
                        }),
                        handle.screen
                    )
                )
            },
            rwh_05::RawDisplayHandle::Wayland(handle) => {
                rwh_06::RawDisplayHandle::Wayland(
                    rwh_06::WaylandDisplayHandle::new(unsafe {
                        NonNull::new_unchecked(handle.display)
                    })
                )
            },
            rwh_05::RawDisplayHandle::Drm(handle) => {
                rwh_06::RawDisplayHandle::Drm(
                    rwh_06::DrmDisplayHandle::new(handle.fd)
                )
            },
            rwh_05::RawDisplayHandle::Gbm(handle) => {
                rwh_06::RawDisplayHandle::Gbm(
                    rwh_06::GbmDisplayHandle::new(unsafe {
                        NonNull::new_unchecked(handle.gbm_device)
                    })
                )
            },
            rwh_05::RawDisplayHandle::Web(_) => {
                rwh_06::RawDisplayHandle::Web(rwh_06::WebDisplayHandle::new())
            },
            rwh_05::RawDisplayHandle::Haiku(_) => {
                rwh_06::RawDisplayHandle::Haiku(
                    rwh_06::HaikuDisplayHandle::new()
                )
            },
            rwh_05::RawDisplayHandle::Android(_) => {
                rwh_06::RawDisplayHandle::Android(
                    rwh_06::AndroidDisplayHandle::new()
                )
            },
            rwh_05::RawDisplayHandle::Windows(_) => {
                rwh_06::RawDisplayHandle::Windows(
                    rwh_06::WindowsDisplayHandle::new()
                )
            }
            _ => panic!("Raw window handle conversion not supported"),
        }
    }
}