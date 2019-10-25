use std::io;
use std::iter;
use std::{slice, time::Duration};

use winapi::shared::usbiodef::{
    GUID_DEVINTERFACE_USB_DEVICE
};

use winapi::um::setupapi::{
    SP_DEVICE_INTERFACE_DATA,
    DIGCF_PRESENT, DIGCF_DEVICEINTERFACE,
    SetupDiGetDeviceInterfaceDetailA,
    SetupDiGetClassDevsA,
};

use std::ptr;

fn get_windows_usb_device_detail() -> io::Result<()> {
    // SP_DEVICE_INTERFACE_DATA
    let device_info_set = unsafe { SetupDiGetClassDevsA(&GUID_DEVINTERFACE_USB_DEVICE, ptr::null(), ptr::null_mut(), DIGCF_PRESENT | DIGCF_DEVICEINTERFACE) };
    println!("device_info: {:?}", device_info_set);
    Ok(())
}

// fn hid_get_feature_report<T: UsbContext>(handle: &mut DeviceHandle<T>, data: &mut [u8]) -> Result<()> {
//     let timeout = Duration::from_secs(10);
//     let report_number = data[0];

//     println!("send data buffer: {:?}", data);
//     match handle.read_control(
//         LIBUSB_REQUEST_TYPE_CLASS|LIBUSB_RECIPIENT_INTERFACE|LIBUSB_ENDPOINT_IN,
//         0x01, // HID get_report
//         u16::from(0x03u8) << 8 | u16::from(report_number),
//         0, // TODO: find this in handle
//         data,
//         timeout) {
//             Ok(_size) => Ok(()),
//             Err(e) => {
//                 println!("oh no! {:?}", data);
//                 Err(e)
//             }
//         }
// }
