pub(crate) mod iter;
pub(crate) mod sys;

use std::fmt::{
    Display, Formatter, Result,
};

#[derive(Default, Debug)]
pub struct HIDDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub release_number: u16,
    pub interface_number: u16,
    pub path: String,
    pub class: String,
    pub driver_name: String,
    pub serial_number: String,
    pub manufacturer_string: String,
    pub product_string: String,
}

impl Display for HIDDeviceInfo {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "vendor_id: {:#06x}, product_id: {:#06x}, path: {}", self.vendor_id, self.product_id, self.path)
    }
}