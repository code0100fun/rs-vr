use std::io;

use winapi::shared::minwindef::{TRUE};
use winapi::shared::usbiodef::{
    GUID_DEVINTERFACE_USB_DEVICE
};
use winapi::um::handleapi::{
    INVALID_HANDLE_VALUE
};
use winapi::um::winnt::{
    HANDLE, LPCSTR,
    GENERIC_READ, GENERIC_WRITE,
    FILE_SHARE_READ, FILE_SHARE_WRITE,
};
use winapi::um::fileapi::{
    CreateFileA,
    OPEN_EXISTING,
};
use winapi::um::winbase::{
    FILE_FLAG_OVERLAPPED,
};

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

use std::ptr;

pub struct Device {

}

pub fn enumerate(_vendor_id: u16, _product_id: u16) -> io::Result<Device> {
    get_windows_usb_device_detail().unwrap();
    Ok(Device {})
}

fn get_windows_usb_device_detail() -> io::Result<()> {
    let device_info_set = get_device_info_set().unwrap();
    let has_hid = has_hid_driver_bound(device_info_set);
    println!("has_hid: {}", has_hid);
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
        // let mut native_detail = device_interface_detail_data._native;
        get_device_detail(device_info_set, &mut device_interface_data, &mut device_interface_detail_data._native, detail_size).unwrap();

        let device_name = bytes_to_str(&device_interface_detail_data.device_path_mem);

        let prepend = vec![92, 92, 63, 92]; // why is this prepended on the DevicePath???
        let append = vec![0]; // terminate with 0
        let mut device_path_bytes = vec![];
        device_path_bytes.extend(prepend);
        device_path_bytes.extend(device_name.as_bytes().to_vec());
        device_path_bytes.extend(append);

        let handle = open_device(device_path_bytes.as_ptr() as *const i8, true).unwrap();
        println!("handle: {:?}", handle);

        device_index += 1;
    }
    Ok(())
}

fn bytes_to_str(bytes: &[u8]) -> &str {
    let first_null = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    &std::str::from_utf8(bytes).unwrap()[0..first_null]
}

fn str_to_os_str(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;

    let os_str: Vec<u16> = OsStr::new(s).encode_wide().chain(once(0)).collect();
    os_str
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

fn has_hid_driver_bound(device_info_set: HDEVINFO) -> bool {
    // iterate device info looking for bound HIDClass
    // https://github.com/signal11/hidapi/blob/a6a622ffb680c55da0de787ff93b80280498330f/windows/hid.c#L345
    let mut has_hid_driver = false;
    for info_index in 0.. {

        let mut driver_name: [u8; 256] = unsafe { std::mem::zeroed() };
        let mut dev_info_data = create_devinfo_data();

        let has_next_info = TRUE == unsafe { SetupDiEnumDeviceInfo(device_info_set, info_index, &mut dev_info_data) };

        if !has_next_info {
            break;
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
            continue;
        }

        let class_name = bytes_to_str(&driver_name);
        println!("device_class: {}", class_name);

        if class_name != "HIDClass" {
            continue;
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
            has_hid_driver = true;
            let driver_name_str = bytes_to_str(&driver_name);
            println!("driver_name: {}", driver_name_str);
            break;
        }
    }

    has_hid_driver
}

fn open_device(device_path: LPCSTR, enumerate: bool) -> io::Result<HANDLE> {
    // println!("open_device: {:?} {:?}", str_to_bytes(device_path), bytes_to_str(str_to_bytes(device_path).as_slice()));
    let desired_access = if enumerate { 0 } else { GENERIC_WRITE | GENERIC_READ };
    let share_mode = FILE_SHARE_READ | FILE_SHARE_WRITE;
    // https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilea
    let handle = unsafe {
        CreateFileA(
            device_path,
            desired_access,
            share_mode,
            ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED,
            ptr::null_mut()
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        Err(io::Error::last_os_error())
    } else {
        Ok(handle)
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
    let mut device_path_mem: [u8; 100] = unsafe { std::mem::zeroed() }; // hardcode 100 for now
    device_interface_detail_data = SP_DEVICE_INTERFACE_DETAIL_DATA_A {
        cbSize: std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_A>() as u32,
        DevicePath: unsafe {
            *std::mem::transmute::<*const i8, &[i8; 1]>(device_path_mem.as_ptr() as *const i8)
        }
    };
    // device_interface_detail_data.cbSize = std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_A>() as u32;

    DeviceInterfaceDetailData {
        _native: device_interface_detail_data,
        device_path_mem: device_path_mem, // the device_path will be stored here
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