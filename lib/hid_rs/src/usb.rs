pub(crate) mod device;

use device::info::iter::HIDDeviceInfoIter;

pub fn hid_enumerate_all() -> HIDDeviceInfoIter {
    HIDDeviceInfoIter {
        index: 0,
        device_info_set: None,
    }
}