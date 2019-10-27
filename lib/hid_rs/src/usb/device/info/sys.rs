use winapi::um::setupapi::{
    SPDRP_CLASS,
    SPDRP_DRIVER,
    HDEVINFO,
    SP_DEVICE_INTERFACE_DATA,
    SP_DEVINFO_DATA,
    DIGCF_PRESENT,
    DIGCF_DEVICEINTERFACE,
    SP_DEVICE_INTERFACE_DETAIL_DATA_A,
    SetupDiGetDeviceInterfaceDetailA,
    SetupDiEnumDeviceInterfaces,
    SetupDiGetClassDevsA,
    SetupDiEnumDeviceInfo,
    SetupDiGetDeviceRegistryPropertyA,
};
use winapi::um::handleapi::{
    INVALID_HANDLE_VALUE,
};
use winapi::shared::hidsdi::{
    HidD_GetAttributes,
    HIDD_ATTRIBUTES,
};
use winapi::shared::guiddef::{GUID};
use winapi::shared::minwindef::{TRUE};

use std::io;
use std::ptr;

use super::{HIDDeviceInfo};
use crate::utils::{
    bytes_to_str, str_to_os_str,
};
use crate::usb::device::{
    open_device, close_device,
};

// page 4 of https://www.ftdichip.com/Support/Documents/AppNotes/AN_152_Detecting_USB_%20Device_Insertion_and_Removal.pdf
const GUID_DEVINTERFACE_USB: GUID = GUID {
    Data1: 0x4d1e55b2,
    Data2: 0xf16f,
    Data3: 0x11cf,
    Data4: [0x88, 0xcb, 0x00, 0x11, 0x11, 0x00, 0x00, 0x30],
};

pub fn get_device_info(device_info_set: HDEVINFO, device_index: u32) -> (bool, Option<io::Result<HIDDeviceInfo>>) {
    let has_next;
    let mut device_interface_data = create_device_interface_data();
    let mut device_interface_detail_data = create_device_interface_detail_data();

    has_next = has_more_devices(device_info_set, device_index, &mut device_interface_data);
    if !has_next {
        return (has_next, None);
    }

    let detail_size = get_device_detail_size(device_info_set, &mut device_interface_data).unwrap();

    get_device_detail(
        device_info_set,
        &mut device_interface_data,
        &mut device_interface_detail_data._native,
        detail_size
    ).unwrap();

    let device_path = device_interface_detail_data.device_path();

    let has_hid = device_has_hid_driver_bound(device_info_set, device_index);
    if !has_hid {
        // this device does not have a HID driver bound
        return (has_next, None);
    }

    let write_handle = match open_device(str_to_os_str(device_path).as_ptr(), true) {
        Ok(handle) if handle == INVALID_HANDLE_VALUE => {
            // could not open device
            let error = io::Error::new(io::ErrorKind::Other, "got invalid handle to device");
            return (has_next, Some(Err(error)))
        }
        Ok(handle) => handle, // success
        Err(error) => return (has_next, Some(Err(error))),
    };

    let mut device_info = HIDDeviceInfo::default();

    let hid_attribs: HIDD_ATTRIBUTES = HIDD_ATTRIBUTES {
        ProductID: 0,
        VendorID: 0,
        VersionNumber: 0,
        Size: std::mem::size_of::<HIDD_ATTRIBUTES>() as u32,
    };
    let mut hid_attribs: HIDD_ATTRIBUTES = unsafe { std::mem::zeroed() };
    hid_attribs.Size = std::mem::size_of::<HIDD_ATTRIBUTES>() as u32;
    unsafe {
        HidD_GetAttributes(write_handle, &mut hid_attribs);
    }
    device_info.vendor_id = hid_attribs.VendorID;
    device_info.product_id = hid_attribs.ProductID;
    device_info.release_number = hid_attribs.VersionNumber;

    close_device(write_handle).unwrap();

    // return device
    (has_next, Some(Ok(device_info)))
}

// ------
// unsafe winapi garbage
// ------

pub fn get_device_info_set() -> io::Result<HDEVINFO> {
    match unsafe {
        SetupDiGetClassDevsA(&GUID_DEVINTERFACE_USB, ptr::null(), ptr::null_mut(), DIGCF_PRESENT | DIGCF_DEVICEINTERFACE)
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
            &GUID_DEVINTERFACE_USB,
            device_index,
            device_interface_data
        )
    }
}


// This will always return false even if there was no OS error
// https://stackoverflow.com/questions/1054748/setupdigetdeviceinterfacedetail-unexplainable-error
// https://docs.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupdigetdeviceinterfacedetaila#remarks
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

// check device for bound HIDClass driver
// https://github.com/signal11/hidapi/blob/a6a622ffb680c55da0de787ff93b80280498330f/windows/hid.c#L345
fn device_has_hid_driver_bound(device_info_set: HDEVINFO, info_index: u32) -> bool {
    let mut driver_name: [u8; 256] = unsafe { std::mem::zeroed() };
    let mut dev_info_data = create_devinfo_data();

    let has_next_info = TRUE == unsafe { SetupDiEnumDeviceInfo(device_info_set, info_index, &mut dev_info_data) };

    if !has_next_info {
        return false
    }

    // get device class
    let result = TRUE == unsafe {
        SetupDiGetDeviceRegistryPropertyA(
            device_info_set,
            &mut dev_info_data,
            SPDRP_CLASS,
            ptr::null_mut(),
            driver_name.as_mut_ptr(),
            std::mem::size_of_val(&driver_name) as u32,
            ptr::null_mut()
        )
    };

    if !result {
        // no device class so skip the name lookup
        return false
    }

    let class_name = bytes_to_str(&driver_name);
    if class_name != "HIDClass" {
        return false
    }

    // get device driver name
    let result = TRUE == unsafe {
        SetupDiGetDeviceRegistryPropertyA(
            device_info_set,
            &mut dev_info_data,
            SPDRP_DRIVER,
            ptr::null_mut(),
            driver_name.as_mut_ptr(),
            std::mem::size_of_val(&driver_name) as u32,
            ptr::null_mut()
        )
    };

    if result {
        // TODO: return this in device_info
        // let driver_name_str = bytes_to_str(&driver_name);
        return true
    } else {
        return false
    }
}

// ------
// My unsafe garbage
// ------

struct DeviceInterfaceDetailData {
    _native: SP_DEVICE_INTERFACE_DETAIL_DATA_A,
    _device_path_mem: [u8; 100], // could actually be 4 bytes smaller but meh
}

impl DeviceInterfaceDetailData {
    fn device_path(&self) -> &str {
        // Remember when we did that thing there...
        let device_path_bytes = unsafe {
            // read all the way through the end of the _native part into the
            // _device_path_mem region
            std::slice::from_raw_parts(self._native.DevicePath.as_ptr() as *const u8, 100)
        };
        bytes_to_str(device_path_bytes)
    }
}

fn create_device_interface_detail_data() -> DeviceInterfaceDetailData {
    let device_interface_detail_data = SP_DEVICE_INTERFACE_DETAIL_DATA_A {
        cbSize: std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_A>() as u32,
        DevicePath: [0],
    };

    // HACK!!! This is a really hacky way to get a chunk of memory for the device detail data
    DeviceInterfaceDetailData {
        _native: device_interface_detail_data,
        _device_path_mem: unsafe { std::mem::zeroed() }, // the device_path will overflow and be stored here
    }
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