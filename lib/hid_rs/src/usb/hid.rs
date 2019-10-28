use winapi::{
    shared::{
        hidclass::{
            IOCTL_HID_GET_FEATURE,
        },
        minwindef::{
            DWORD,
            TRUE,
            LPVOID,
        },
        winerror::{
            ERROR_IO_PENDING,
        },
    },
    um::{
        ioapiset::{
            DeviceIoControl,
            GetOverlappedResult,
        },
        minwinbase::{
            OVERLAPPED,
        },
        winnt::{
            HANDLE,
        }
    },
};
use std::io;

pub fn hid_get_feature_report(handle: HANDLE, data: &mut [u8]) -> io::Result<(u32)> {
    let mut overlapped = OVERLAPPED::default();
    let mut bytes_returned: DWORD = 0;

    if TRUE == unsafe {
        DeviceIoControl(
            handle,
            IOCTL_HID_GET_FEATURE,
            data.as_mut_ptr() as LPVOID, data.len() as u32,
            data.as_mut_ptr() as LPVOID, data.len() as u32,
            &mut bytes_returned,
            &mut overlapped
        )
    } {
        if TRUE == unsafe {
            // wait for result
            GetOverlappedResult(
                handle,
                &mut overlapped,
                &mut bytes_returned,
                TRUE // wait
            )
        } {
            Ok(bytes_returned)
        } else {
            Err(io::Error::last_os_error())
        }
    } else {
        Err(io::Error::last_os_error())
    }
}