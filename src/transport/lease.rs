//! Named system mutex. Vendor software that drives the same device takes a mutex of a known name;
//! holding it for the duration of one operation keeps the two programs from interleaving packets.

use crate::core::{DriverError, Guard};

#[cfg(windows)]
mod imp {
    use super::{DriverError, Guard};
    use std::ffi::c_void;

    const WAIT_OBJECT_0: u32 = 0;
    const WAIT_ABANDONED: u32 = 0x80;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateMutexW(attributes: *const c_void, owner: i32, name: *const u16) -> *mut c_void;
        fn WaitForSingleObject(handle: *mut c_void, milliseconds: u32) -> u32;
        fn ReleaseMutex(handle: *mut c_void) -> i32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }

    pub(super) struct DeviceLease(*mut c_void);

    impl Guard for DeviceLease {}

    pub(super) fn acquire(name: &str) -> Result<DeviceLease, DriverError> {
        let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        // SAFETY: `wide` is NUL-terminated and outlives the call; null attributes are allowed.
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, wide.as_ptr()) };
        if handle.is_null() {
            return Err(DriverError::Io("CreateMutexW failed".into()));
        }
        // SAFETY: `handle` is a valid mutex handle created above; a zero timeout never blocks.
        let waited = unsafe { WaitForSingleObject(handle, 0) };
        if waited != WAIT_OBJECT_0 && waited != WAIT_ABANDONED {
            // SAFETY: the handle is valid and not used after this point.
            unsafe { CloseHandle(handle) };
            return Err(DriverError::Busy);
        }
        Ok(DeviceLease(handle))
    }

    impl Drop for DeviceLease {
        fn drop(&mut self) {
            // SAFETY: the mutex is owned by this thread (taken in `acquire`, and the guard is not
            // `Send`), and the handle is closed exactly once.
            unsafe {
                ReleaseMutex(self.0);
                CloseHandle(self.0);
            }
        }
    }
}

#[cfg(windows)]
pub(super) fn acquire(name: &str) -> Result<Box<dyn Guard>, DriverError> {
    Ok(Box::new(imp::acquire(name)?))
}

#[cfg(not(windows))]
pub(super) fn acquire(_name: &str) -> Result<Box<dyn Guard>, DriverError> {
    struct Unlocked;
    impl Guard for Unlocked {}
    Ok(Box::new(Unlocked))
}
