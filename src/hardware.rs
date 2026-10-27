use anyhow::{Result, ensure};
use std::ffi::c_void;

pub(crate) struct DeviceLease(*mut c_void);
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateMutexW(attributes: *const c_void, owner: i32, name: *const u16) -> *mut c_void;
    fn WaitForSingleObject(handle: *mut c_void, milliseconds: u32) -> u32;
    fn ReleaseMutex(handle: *mut c_void) -> i32;
    fn CloseHandle(handle: *mut c_void) -> i32;
}
impl DeviceLease {
    pub(crate) fn acquire(model: &str) -> Result<Self> {
        let name: Vec<_> = format!("Local\\AjazzStudio-{model}\0")
            .encode_utf16()
            .collect();
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
        ensure!(
            !handle.is_null(),
            "Не удалось создать блокировку устройства"
        );
        let waited = unsafe { WaitForSingleObject(handle, 0) };
        if waited != 0 && waited != 0x80 {
            unsafe { CloseHandle(handle) };
            anyhow::bail!("Клавиатура занята другой программой");
        }
        Ok(Self(handle))
    }
}
impl Drop for DeviceLease {
    fn drop(&mut self) {
        unsafe {
            ReleaseMutex(self.0);
            CloseHandle(self.0);
        }
    }
}
