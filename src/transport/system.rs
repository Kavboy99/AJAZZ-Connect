//! Windows user preferences the interface must follow.

#[cfg(windows)]
#[link(name = "user32")]
unsafe extern "system" {
    fn SystemParametersInfoW(action: u32, param: u32, value: *mut i32, update: u32) -> i32;
}

/// "Show animations in Windows". When off, motion drops displacement and scaling.
#[cfg(windows)]
pub fn animations_enabled() -> bool {
    const SPI_GETCLIENTAREAANIMATION: u32 = 0x1042;
    let mut enabled: i32 = 1;
    // SAFETY: for this action Windows writes one BOOL into the pointer, which is valid for the
    // duration of the call.
    let ok = unsafe { SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, &mut enabled, 0) };
    ok == 0 || enabled != 0
}

#[cfg(not(windows))]
pub fn animations_enabled() -> bool {
    true
}
