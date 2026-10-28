//! Notification when a HID collection appears or disappears. Windows calls back on its own thread
//! pool (`CM_Register_Notification`), so the application keeps no thread, timer or polling loop
//! for it: an idle machine costs nothing.

/// Called on every HID arrival or removal. Must be cheap and must not block.
pub type OnChange = Box<dyn Fn() + Send + Sync>;

#[cfg(windows)]
mod imp {
    use super::OnChange;
    use std::ffi::c_void;

    const CR_SUCCESS: u32 = 0;
    const CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE: u32 = 0;
    const CM_NOTIFY_ACTION_DEVICEINTERFACEARRIVAL: u32 = 0;
    const CM_NOTIFY_ACTION_DEVICEINTERFACEREMOVAL: u32 = 1;
    /// `MAX_DEVICE_ID_LEN` UTF-16 units: the size of the filter's union.
    const UNION_UNITS: usize = 200;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    /// `GUID_DEVINTERFACE_HID`.
    const HID_INTERFACE: Guid = Guid {
        data1: 0x4d1e_55b2,
        data2: 0xf16f,
        data3: 0x11cf,
        data4: [0x88, 0xcb, 0x00, 0x11, 0x11, 0x00, 0x00, 0x30],
    };

    /// `CM_NOTIFY_FILTER` with the device-interface member of its union.
    #[repr(C)]
    struct Filter {
        size: u32,
        flags: u32,
        kind: u32,
        reserved: u32,
        class: Guid,
        rest: [u16; UNION_UNITS - 8],
    }

    type Callback = unsafe extern "system" fn(
        notify: *mut c_void,
        context: *mut c_void,
        action: u32,
        data: *const c_void,
        size: u32,
    ) -> u32;

    #[link(name = "cfgmgr32")]
    unsafe extern "system" {
        fn CM_Register_Notification(
            filter: *const Filter,
            context: *mut c_void,
            callback: Callback,
            notify: *mut *mut c_void,
        ) -> u32;
        fn CM_Unregister_Notification(notify: *mut c_void) -> u32;
    }

    unsafe extern "system" fn on_change(
        _notify: *mut c_void,
        context: *mut c_void,
        action: u32,
        _data: *const c_void,
        _size: u32,
    ) -> u32 {
        if matches!(
            action,
            CM_NOTIFY_ACTION_DEVICEINTERFACEARRIVAL | CM_NOTIFY_ACTION_DEVICEINTERFACEREMOVAL
        ) {
            // SAFETY: `context` is the `OnChange` boxed in `DeviceWatch`, which outlives the
            // registration: `Drop` unregisters before the box is freed.
            let handler = unsafe { &*(context as *const OnChange) };
            handler();
        }
        0
    }

    pub struct DeviceWatch {
        handle: *mut c_void,
        // Read only through the raw pointer given to Windows; kept alive here.
        _handler: Box<OnChange>,
    }

    // SAFETY: the handle is only passed back to `CM_Unregister_Notification`, which may be called
    // from any thread; the handler itself is `Send + Sync`.
    unsafe impl Send for DeviceWatch {}

    impl DeviceWatch {
        pub fn start(handler: OnChange) -> Option<Self> {
            let handler = Box::new(handler);
            let filter = Filter {
                size: std::mem::size_of::<Filter>() as u32,
                flags: 0,
                kind: CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE,
                reserved: 0,
                class: HID_INTERFACE,
                rest: [0; UNION_UNITS - 8],
            };
            let mut handle = std::ptr::null_mut();
            let context = &*handler as *const OnChange as *mut c_void;
            // SAFETY: `filter` and `handle` are valid for the call; `context` points to the boxed
            // handler that lives as long as the registration (see `Drop`).
            let result =
                unsafe { CM_Register_Notification(&filter, context, on_change, &mut handle) };
            (result == CR_SUCCESS && !handle.is_null()).then_some(Self {
                handle,
                _handler: handler,
            })
        }
    }

    impl Drop for DeviceWatch {
        fn drop(&mut self) {
            // SAFETY: the handle came from a successful registration and is released once; Windows
            // waits for running callbacks, so the handler is not used after this returns.
            unsafe { CM_Unregister_Notification(self.handle) };
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn filter_has_the_size_windows_expects() {
            // cbSize of CM_NOTIFY_FILTER: four DWORDs and a 200-unit WCHAR union.
            assert_eq!(std::mem::size_of::<super::Filter>(), 16 + 400);
        }
    }
}

#[cfg(windows)]
pub use imp::DeviceWatch;

/// Without Windows there is nothing to watch; the interface keeps the manual refresh.
#[cfg(not(windows))]
pub struct DeviceWatch;

#[cfg(not(windows))]
impl DeviceWatch {
    pub fn start(_handler: OnChange) -> Option<Self> {
        None
    }
}
