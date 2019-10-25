use std::io;

use winapi::shared::minwindef::{TRUE};
use winapi::shared::usbiodef::{
    GUID_DEVINTERFACE_USB_DEVICE
};
use winapi::um::handleapi::{
    INVALID_HANDLE_VALUE
};

use winapi::um::setupapi::{
    HDEVINFO,
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

pub fn enumerate(_vendor_id: u16, _product_id: u16) -> io::Result<Device> {
    get_windows_usb_device_detail().unwrap();
    Ok(Device {})
}

fn get_windows_usb_device_detail() -> io::Result<()> {
    let device_info_set = get_device_info_set().unwrap();
    let mut device_index = 0;
    let mut has_next = true;
    while has_next {
        let mut device_interface_data = create_device_interface_data();
        let mut device_interface_detail_data = create_device_interface_detail_data();

        has_next = has_more_devices(device_info_set, device_index, &mut device_interface_data);

        if !has_next {
            break;
        }

        let detail_size = get_device_detail_size(device_info_set, &mut device_interface_data).unwrap();
        get_device_detail(device_info_set, &mut device_interface_data, &mut device_interface_detail_data._native, detail_size).unwrap();

        let device_path = device_path_for_data(&device_interface_detail_data);

        println!("detail_size: {}, details: {:?}", detail_size, device_path);

        device_index += 1;
    }
    Ok(())
}


fn bytes_to_str(bytes: &[u8]) -> &str {
    let first_null = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    &std::str::from_utf8(bytes).unwrap()[0..first_null]
}

// ------
// unsafe winapi garbage
// ------

fn get_device_info_set() -> io::Result<HDEVINFO> {
    match unsafe {
        SetupDiGetClassDevsA(&GUID_DEVINTERFACE_USB_DEVICE, ptr::null(), ptr::null_mut(), DIGCF_PRESENT | DIGCF_DEVICEINTERFACE)
    } {
        d if d == INVALID_HANDLE_VALUE => Err(io::Error::last_os_error()),
        device_info_set => Ok(device_info_set),
    }
}

fn has_more_devices(device_info_set: HDEVINFO, device_index: u32, device_interface_data: &mut SP_DEVICE_INTERFACE_DATA) -> bool {
    TRUE == unsafe {
        SetupDiEnumDeviceInterfaces(
            device_info_set,
            ptr::null_mut(),
            &GUID_DEVINTERFACE_USB_DEVICE,
            device_index,
            device_interface_data
        )
    }
}

// This will always return false even if there was no OS error
// https://stackoverflow.com/questions/1054748/setupdigetdeviceinterfacedetail-unexplainable-error
fn get_device_detail_size(device_info_set: HDEVINFO, device_interface_data: &mut SP_DEVICE_INTERFACE_DATA) -> io::Result<u32> {
    let mut detail_size: u32 = 0;
    unsafe {
        SetupDiGetDeviceInterfaceDetailA(
            device_info_set,
            device_interface_data,
            ptr::null_mut(),
            0,
            &mut detail_size, // store size here
            ptr::null_mut()
        )
    };

    if detail_size > 0 {
        Ok(detail_size)
    } else {
        Err(io::Error::last_os_error())
    }
}

fn get_device_detail(
    device_info_set: HDEVINFO,
    device_interface_data: &mut SP_DEVICE_INTERFACE_DATA,
    device_interface_detail_data: &mut SP_DEVICE_INTERFACE_DETAIL_DATA_A,
    detail_size: u32
    ) -> io::Result<()> {
    let result = TRUE == unsafe {
        SetupDiGetDeviceInterfaceDetailA(
            device_info_set,
            device_interface_data,
            device_interface_detail_data, // store details here
            detail_size,
            ptr::null_mut(),
            ptr::null_mut()
        )
    };

    if result {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

// ------
// My unsafe garbage
// ------

struct DeviceInterfaceDetailData {
    _native: SP_DEVICE_INTERFACE_DETAIL_DATA_A,
    device_path_mem: [u8; 100],
}

fn create_device_interface_detail_data() -> DeviceInterfaceDetailData {
    let mut device_interface_detail_data: SP_DEVICE_INTERFACE_DETAIL_DATA_A;
    // HACK!!! This is a really hacky way to get a chunk of memory for the device detail data
    let device_path_mem: [u8; 100] = unsafe { std::mem::zeroed() }; // hardcode 100 for now
    device_interface_detail_data = unsafe {
        std::mem::transmute(&device_path_mem) // the device_path will be stored here
    };
    device_interface_detail_data.cbSize = std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_A>() as u32;

    DeviceInterfaceDetailData {
        _native: device_interface_detail_data,
        device_path_mem: device_path_mem,
    }
}

fn device_path_for_data(device_interface_detail_data: &DeviceInterfaceDetailData) -> &str {
    // Remember when we did that thing up there...
    bytes_to_str(&device_interface_detail_data.device_path_mem)
}

fn create_device_interface_data() -> SP_DEVICE_INTERFACE_DATA {
    let mut device_interface_data: SP_DEVICE_INTERFACE_DATA = unsafe { std::mem::zeroed() };
    device_interface_data.cbSize = std::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32;
    device_interface_data
}

fn create_devinfo_data() -> SP_DEVINFO_DATA {
    let mut devinfo_data: SP_DEVINFO_DATA = unsafe { std::mem::zeroed() };
    devinfo_data.cbSize = std::mem::size_of::<SP_DEVINFO_DATA>() as u32;
    devinfo_data
}