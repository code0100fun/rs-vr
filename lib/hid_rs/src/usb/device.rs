pub(crate) mod info;

use std::io;

use winapi::shared::minwindef::{TRUE};
use winapi::um::handleapi::{
    INVALID_HANDLE_VALUE,
    CloseHandle
};
use winapi::um::winnt::{
    HANDLE, LPCSTR,
    GENERIC_READ, GENERIC_WRITE,
    FILE_SHARE_READ, FILE_SHARE_WRITE,
};
use winapi::um::fileapi::{
    CreateFileA,
    OPEN_EXISTING,
};
use winapi::um::winbase::{
    FILE_FLAG_OVERLAPPED,
};

use std::ptr;

#[derive(Debug)]
pub struct HIDDevice {
    pub handle: HANDLE,
}

pub fn open_device(device_path: LPCSTR, enumerate: bool) -> io::Result<HANDLE> {
    let desired_access = if enumerate { 0 } else { GENERIC_WRITE | GENERIC_READ };
    // https://github.com/signal11/hidapi/commit/b5b2e1779b6cd2edda3066bbbf0921a2d6b1c3c0
    let share_mode = FILE_SHARE_READ | FILE_SHARE_WRITE;
    // https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilea
    let handle = unsafe {
        CreateFileA(
            device_path,
            desired_access,
            share_mode,
            ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED,
            ptr::null_mut()
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        Err(io::Error::last_os_error())
    } else {
        Ok(handle)
    }
}

fn close_device(handle: HANDLE) -> io::Result<()> {
    let result = TRUE == unsafe { CloseHandle(handle) };
    if result {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
