//! No-op stand-ins for two private CoreGraphics functions.
//!
//! winit's `set_blur` calls `CGSMainConnectionID` and
//! `CGSSetWindowBackgroundBlurRadius`. App Store review rejects binaries that
//! import non-public symbols, and SweepLoom never blurs a window, so the App
//! Store build defines both here: the linker binds winit's calls to these
//! definitions instead of importing them from CoreGraphics.

use std::ffi::c_void;

/// Stand-in for the private `CGSMainConnectionID`; there is no connection.
#[allow(unsafe_code, non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn CGSMainConnectionID() -> *mut c_void {
    std::ptr::null_mut()
}

/// Stand-in for the private `CGSSetWindowBackgroundBlurRadius`; reports success
/// without blurring.
#[allow(unsafe_code, non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn CGSSetWindowBackgroundBlurRadius(
    _connection: *mut c_void,
    _window: isize,
    _radius: i64,
) -> i32 {
    0
}
