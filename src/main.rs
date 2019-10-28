mod controller;
mod bluetooth;
mod utils;

use hid_rs::usb::{
    hid_enumerate_all,
    hid_open_path,
};
use crate::controller::ps_move::{
    PS_MOVE_VID, PS_MOVE_PID,
    get_controller_pair,
};
use crate::bluetooth::{get_host_address};

fn main() {
    // get computer's bluetooth radio MAC
    let host_addr = get_host_address().unwrap();
    println!("bluetooth host_addr: {:?}", host_addr);
    // find PS Move controller
    let mut device = None;
    // https://github.com/psmoveservice/PSMoveService/blob/edbb31417/src/psmoveservice/PSMoveController/PSMoveController.cpp#L1057
    for result in hid_enumerate_all()
        .filter(|d| d.is_ok())
        .filter(|d| {
            let d = d.as_ref().unwrap();
            d.vendor_id == PS_MOVE_VID &&
            d.product_id == PS_MOVE_PID &&
            d.path.contains("&col02#")
         }) {
        match result {
            Ok(device_info) => {
                device = Some(hid_open_path(&device_info.path));
                break
            }
            Err(error) => panic!(error),
        }
    };

    let device = device.unwrap().unwrap();
    let (cur_host_addr, controller_addr) = get_controller_pair(&device).unwrap();
    println!("cur_host_addr: {:?}, controller_addr: {:?}", cur_host_addr, controller_addr);

    // send radio MAC to controller (pair)

    // connect to controllers BT addr
    // start reading position data
}