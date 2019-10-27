mod usb;
mod controller;

use controller::ps_move::{PS_MOVE_VID, PS_MOVE_PID};

fn main() {
    // get computer's bluetooth radio MAC
    // find PS Move controller
    for result in usb::device::hid_enumerate_all() {
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