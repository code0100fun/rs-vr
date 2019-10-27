use winapi::um::setupapi::{
    HDEVINFO,
};

use std::io;

use super::{HIDDeviceInfo};
use super::sys::{
    get_device_info_set,
    get_device_info,
};

pub struct HIDDeviceInfoIter {
    pub index: u32,
    pub device_info_set: Option<HDEVINFO>,
}

impl Iterator for HIDDeviceInfoIter {
    type Item = io::Result<HIDDeviceInfo>;

    fn next(&mut self) -> Option<io::Result<HIDDeviceInfo>> {
        match self.device_info_set() {
            Ok(device_info_set) => {
                match get_next_hid_device_info(device_info_set, self.index) {
                    (false, _, _) => {
                        None // no more results
                    }
                    (_, index, Some(Ok(device_info))) => {
                        self.index = index + 1; // may have skipped over non-HID devices
                        Some(Ok(device_info))
                    },
                    (_, index, Some(Err(error))) => {
                        self.index = index + 1; // may have skipped over non-HID devices
                        Some(Err(error)) // an error, return it but keep going
                    },
                    _ => unreachable!(),
                }
            }
            Err(_) => None, // can't iterate if we can't get this
        }
    }
}

impl HIDDeviceInfoIter {
    fn device_info_set(&mut self) -> io::Result<HDEVINFO> {
        match self.device_info_set {
            None => {
                match get_device_info_set() {
                    Ok(device_info_set) => {
                        self.device_info_set = Some(device_info_set);
                        // fetch succeess
                        Ok(device_info_set)
                    }
                    Err(err) => Err(err),
                }
            }
            Some(device_info_set) => Ok(device_info_set) // cache success
        }
    }
}

fn get_next_hid_device_info(device_info_set: HDEVINFO, index: u32) -> (bool, u32, Option<io::Result<HIDDeviceInfo>>) {
    // get device at index
    // open and get info
    // is it the correct vendor/product
    // no then skip and try again
    // yes then build result
    let mut curr_index = index;
    let result;
    loop {
        match get_device_info(device_info_set, curr_index) {
            (true, None) => {
                curr_index += 1; // not a HID device so go to the next one
            }
            (true, Some(ok_or_error)) => {
                result = (true, curr_index, Some(ok_or_error)); // we found a HID device
                break;
            }
            (false, None) => {
                result = (false, curr_index, None); // no more devices
                break;
            }
            _ => unreachable!(),
        }
    }
    result
}