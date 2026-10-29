//! Native window material and caption contrast for the application's selected theme.
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

pub fn apply(window: &impl HasWindowHandle, dark: bool) -> bool {
    let Ok(handle) = window.window_handle() else {
        return false;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return false;
    };
    crate::transport::acrylic::apply(handle.hwnd.get(), dark)
}
