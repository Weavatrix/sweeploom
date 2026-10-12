//! Real home folder via `getpwuid_r` (inside the sandbox `$HOME` is the container)
//! and integer `sysctl` values without spawning `/usr/sbin/sysctl`.

use std::ffi::{CStr, CString, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

/// Largest buffer tried for one password entry.
const MAX_BUFFER: usize = 1 << 20;

#[allow(unsafe_code)]
pub(crate) fn real_home() -> Option<PathBuf> {
    // SAFETY: getuid has no preconditions and cannot fail.
    let uid = unsafe { libc::getuid() };
    let mut buffer = vec![0 as libc::c_char; 4096];
    loop {
        // SAFETY: an all-zero `passwd` is a valid value (null pointers, zero ids);
        // getpwuid_r overwrites it before we read any field.
        let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
        let mut found: *mut libc::passwd = std::ptr::null_mut();
        // SAFETY: every pointer is valid and writable for the call; `buffer.len()`
        // is the true capacity of `buffer`, which outlives every string read below.
        let code = unsafe {
            libc::getpwuid_r(
                uid,
                &mut entry,
                buffer.as_mut_ptr(),
                buffer.len(),
                &mut found,
            )
        };
        if code == libc::ERANGE && buffer.len() < MAX_BUFFER {
            buffer.resize(buffer.len() * 2, 0);
            continue;
        }
        if code != 0 || found.is_null() || entry.pw_dir.is_null() {
            return None;
        }
        // SAFETY: on success `pw_dir` points at a NUL-terminated string inside
        // `buffer`, which is still alive and unmodified here.
        let dir = unsafe { CStr::from_ptr(entry.pw_dir) };
        let path = PathBuf::from(OsStr::from_bytes(dir.to_bytes()));
        return path.is_absolute().then_some(path);
    }
}

#[allow(unsafe_code)]
pub(crate) fn sysctl_u32(name: &str) -> Option<u32> {
    let name = CString::new(name).ok()?;
    let mut value: u32 = 0;
    let mut size = std::mem::size_of::<u32>();
    // SAFETY: `name` is NUL-terminated; `value`/`size` describe a writable u32
    // and are only read after a successful call that wrote exactly `size` bytes.
    let code = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            (&raw mut value).cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    (code == 0 && size == std::mem::size_of::<u32>()).then_some(value)
}
