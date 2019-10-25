use std::io;
use std::iter;
use std::{slice, time::Duration};

use std::ffi::OsStr;
use winapi::shared::minwindef::{TRUE};
use winapi::shared::usbiodef::{
    GUID_DEVINTERFACE_USB_DEVICE
};

use winapi::um::setupapi::{
    SP_DEVICE_INTERFACE_DATA,
    SP_DEVINFO_DATA,
    DIGCF_PRESENT,
    DIGCF_DEVICEINTERFACE,
    SP_DEVICE_INTERFACE_DETAIL_DATA_A,
    SetupDiGetDeviceInterfaceDetailA,
    SetupDiEnumDeviceInterfaces,
    SetupDiGetClassDevsA,
};

use std::ptr;

pub struct Device {

}

pub fn enumerate(vendor_id: u16, product_id: u16) -> io::Result<Device> {
    get_windows_usb_device_detail();
    Ok(Device {})
}

fn get_windows_usb_device_detail() -> io::Result<()> {
    let device_info_set = unsafe { SetupDiGetClassDevsA(&GUID_DEVINTERFACE_USB_DEVICE, ptr::null(), ptr::null_mut(), DIGCF_PRESENT | DIGCF_DEVICEINTERFACE) };
    println!("device_info: {:?}", device_info_set);

    let mut device_index = 0;
    let mut has_next = true;
    let mut device_interface_data: SP_DEVICE_INTERFACE_DATA = unsafe { std::mem::zeroed() };
    device_interface_data.cbSize = std::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32;
    let mut devinfo_data: SP_DEVINFO_DATA = unsafe { std::mem::zeroed() };
    devinfo_data.cbSize = std::mem::size_of::<SP_DEVINFO_DATA>() as u32;
    let mut device_interface_detail_data: SP_DEVICE_INTERFACE_DETAIL_DATA_A;

    while has_next {
        let mut detail_size = 0;

        has_next = TRUE == unsafe {
            SetupDiEnumDeviceInterfaces(
                device_info_set,
                ptr::null_mut(),
                &GUID_DEVINTERFACE_USB_DEVICE,
                device_index,
                &mut device_interface_data
            )
        };
        if !has_next {
            break;
        }

        // get the size of the detail data
        let result = TRUE == unsafe {
            SetupDiGetDeviceInterfaceDetailA(
                device_info_set,
                &mut device_interface_data,
                ptr::null_mut(),
                0,
                &mut detail_size, // store size here
                ptr::null_mut()
            )
        };

        let device_path_mem: [u8; 100] = unsafe { std::mem::zeroed() }; // hardcode 100 for now
        device_interface_detail_data = unsafe {
            std::mem::transmute(&device_path_mem) // the device_path will be stored here
        };
        device_interface_detail_data.cbSize = std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_A>() as u32;

        // the detail_size gets corrupted after the next call to SetupDiGetDeviceInterfaceDetailA o.O
        let detail_size_saved = detail_size as usize;

        let result = TRUE == unsafe {
            SetupDiGetDeviceInterfaceDetailA(
                device_info_set,
                &mut device_interface_data,
                &mut device_interface_detail_data, // store details here
                detail_size,
                ptr::null_mut(),
                ptr::null_mut()
            )
        };

        let device_path = bytes_to_str(&device_path_mem);

        println!("detail_size_saved: {}, detail_size: {}, details: {:?}", detail_size_saved, detail_size, device_path);

        device_index += 1;
    }
    Ok(())
}

fn bytes_to_str(bytes: &[u8]) -> &str {
    let first_null = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    &std::str::from_utf8(bytes).unwrap()[0..first_null]
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
