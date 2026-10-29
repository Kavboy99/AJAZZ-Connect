//! One DWM material shared by the native caption and transparent navigation surface.
use windows_sys::Win32::{
    Foundation::{GetLastError, HWND, RECT, SetLastError},
    Graphics::{
        Dwm::{
            DWM_BB_BLURREGION, DWM_BB_ENABLE, DWM_BLURBEHIND, DWMSBT_TRANSIENTWINDOW,
            DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_USE_IMMERSIVE_DARK_MODE, DwmEnableBlurBehindWindow,
            DwmExtendFrameIntoClientArea, DwmSetWindowAttribute,
        },
        Gdi::{BLACK_BRUSH, FillRect, GetDC, GetStockObject, ReleaseDC},
    },
    UI::{
        Controls::MARGINS,
        WindowsAndMessaging::{GCLP_HBRBACKGROUND, GetClientRect, SetClassLongPtrW},
    },
};

pub fn apply(handle: isize, dark: bool) -> bool {
    let hwnd = handle as HWND;
    let dark = i32::from(dark);
    let material = DWMSBT_TRANSIENTWINDOW;
    let margins = MARGINS {
        cxLeftWidth: -1,
        cxRightWidth: -1,
        cyTopHeight: -1,
        cyBottomHeight: -1,
    };
    // SAFETY: HWND belongs to the live eframe window. Each attribute points to an i32
    // of the required size, and all argument storage lives through the synchronous calls.
    unsafe {
        // Winit installs an empty legacy region. Enable alpha over the entire HWND instead:
        // disabling it leaves an opaque redirection bitmap under the composition visual.
        let blur = DWM_BLURBEHIND {
            dwFlags: DWM_BB_ENABLE | DWM_BB_BLURREGION,
            fEnable: 1,
            ..Default::default()
        };
        if DwmEnableBlurBehindWindow(hwnd, &blur) < 0 {
            return false;
        }
        if DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
            (&dark as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        ) < 0
        {
            return false;
        }
        if DwmExtendFrameIntoClientArea(hwnd, &margins) < 0 {
            return false;
        }
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE as u32,
            (&material as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        ) >= 0
            && clear_client(hwnd)
    }
}

fn clear_client(hwnd: HWND) -> bool {
    // SAFETY: The HWND is live on this thread. The stock brush is borrowed, never deleted.
    // The acquired client DC is released after the fill, including when drawing fails.
    unsafe {
        let brush = GetStockObject(BLACK_BRUSH);
        if brush.is_null() {
            return false;
        }
        SetLastError(0);
        if SetClassLongPtrW(hwnd, GCLP_HBRBACKGROUND, brush as isize) == 0 && GetLastError() != 0 {
            return false;
        }
        let mut client = RECT::default();
        if GetClientRect(hwnd, &mut client) == 0 {
            return false;
        }
        let dc = GetDC(hwnd);
        if dc.is_null() {
            return false;
        }
        // DWM glass requires zero pixels in the HWND redirection surface. Otherwise its
        // white GDI background covers transparent DX12 pixels. The class brush also keeps
        // newly exposed regions transparent on resize; GPU content is composed separately.
        let filled = FillRect(dc, &client, brush) != 0;
        ReleaseDC(hwnd, dc);
        filled
    }
}
