mod controller;
mod bluetooth;

use controller::ps_move::{PS_MOVE_VID, PS_MOVE_PID};
use crate::bluetooth::{get_host_address};

fn main() {
    // get computer's bluetooth radio MAC
    let host_addr = get_host_address().unwrap();
    println!("bluetooth host_addr: {:?}", host_addr);
    // find PS Move controller
    for result in hid_rs::usb::hid_enumerate_all()
        .filter(|d| d.is_ok())
        .filter(|d| d.as_ref().unwrap().vendor_id == PS_MOVE_VID &&
                    d.as_ref().unwrap().product_id == PS_MOVE_PID) {
        match result {
            Ok(device_info) => {
                println!("{}", device_info)
            }
            Err(error) => panic!(error),
        }
    }

    // send radio MAC to controller (pair)
    // connect to controllers BT addr
    // start reading position data
}