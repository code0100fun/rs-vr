pub mod device;
pub mod hid;

use device::info::iter::HIDDeviceInfoIter;
use device::{
    HIDDevice,
    open_device,
};
use crate::utils::{
    str_to_os_str,
};

use std::io;

pub fn hid_enumerate_all() -> HIDDeviceInfoIter {
    HIDDeviceInfoIter {
        index: 0,
        device_info_set: None,
    }
}

pub fn hid_open_path(device_path: &String) -> io::Result<HIDDevice> {
    match open_device(str_to_os_str(device_path).as_ptr(), true) {
        Ok(handle) => {
            let device = HIDDevice {
                handle: handle,
            };
            Ok(device)
        }
        Err(error) => Err(error),
    }
}