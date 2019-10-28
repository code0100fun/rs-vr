use hid_rs::usb::device::{
    HIDDevice,
};
use hid_rs::usb::hid::{
    hid_get_feature_report,
};

use std::io;

use crate::utils::{
    address_bytes_to_string,
};

pub const PS_MOVE_VID: u16 = 0x054c;
pub const PS_MOVE_PID: u16 = 0x03d5; // PSMove ZCM1
pub const PSMOVE_BTADDR_GET_ZCM1_SIZE: usize = 16;
pub const PSMOVE_BTADDR_GET_ZCM2_SIZE: usize = 21;
pub const PSMOVE_BTADDR_GET_MAX_SIZE: usize = PSMOVE_BTADDR_GET_ZCM2_SIZE;

pub enum PSMoveRequestType {
    GetBTAddr = 0x04,
}

pub fn get_controller_pair(device: &HIDDevice) -> io::Result<(String, String)> {
    let mut data = vec![0u8; PSMOVE_BTADDR_GET_MAX_SIZE];
    data[0] = PSMoveRequestType::GetBTAddr as u8;
    hid_get_feature_report(device.handle, &mut data).unwrap();
    let mut cont_addr = Vec::from(&data[1..7]);
    cont_addr.reverse();
    let mut host_addr = Vec::from(&data[10..16]);
    host_addr.reverse();

    Ok((
        address_bytes_to_string(host_addr.as_slice()),
        address_bytes_to_string(cont_addr.as_slice())
    ))
}