//! `HidP_GetCaps` through the HID class driver. Opens the collection with zero access rights, so it
//! works for keyboards and mice that Windows keeps exclusive, and sends nothing to the device.

use crate::core::ReportSizes;
use std::ffi::c_void;

#[repr(C)]
#[derive(Default)]
struct HidpCaps {
    usage: u16,
    usage_page: u16,
    input: u16,
    output: u16,
    feature: u16,
    reserved: [u16; 17],
    counts: [u16; 10],
}

const HIDP_STATUS_SUCCESS: i32 = 0x0011_0000;
const FILE_SHARE_READ_WRITE: u32 = 3;
const OPEN_EXISTING: u32 = 3;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateFileW(
        path: *const u16,
        access: u32,
        share: u32,
        security: *const c_void,
        creation: u32,
        flags: u32,
        template: *mut c_void,
    ) -> *mut c_void;
    fn CloseHandle(handle: *mut c_void) -> i32;
}

#[link(name = "hid", kind = "raw-dylib")]
unsafe extern "system" {
    fn HidD_GetPreparsedData(handle: *mut c_void, data: *mut *mut c_void) -> u8;
    fn HidD_FreePreparsedData(data: *mut c_void) -> u8;
    fn HidP_GetCaps(data: *mut c_void, caps: *mut HidpCaps) -> i32;
}

/// Report sizes of the collection at `path`, or `None` when Windows does not expose them.
pub(super) fn report_sizes(path: &str) -> Option<ReportSizes> {
    let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    // SAFETY: `wide` is a NUL-terminated UTF-16 string that outlives the call; the remaining
    // arguments are plain flags and null pointers that CreateFileW accepts.
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0,
            FILE_SHARE_READ_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    };
    if handle.is_null() || handle as isize == -1 {
        return None;
    }
    let mut data = std::ptr::null_mut();
    let mut caps = HidpCaps::default();
    // SAFETY: `handle` is a valid open handle; `data` receives a buffer owned by the HID driver
    // that is freed below exactly once; `caps` is a properly sized `#[repr(C)]` HIDP_CAPS.
    let status = unsafe {
        let status = if HidD_GetPreparsedData(handle, &mut data) != 0 {
            HidP_GetCaps(data, &mut caps)
        } else {
            0
        };
        if !data.is_null() {
            HidD_FreePreparsedData(data);
        }
        CloseHandle(handle);
        status
    };
    (status == HIDP_STATUS_SUCCESS).then_some(ReportSizes {
        input: caps.input,
        output: caps.output,
        feature: caps.feature,
    })
}
