use std::io;
use std::iter;
use std::{slice, time::Duration};

use io_bluetooth::bt::{self, BtStream};
use rusb::{
    Context, Device, DeviceDescriptor, DeviceHandle, Direction, Result, TransferType, UsbContext,
};
use libusb1_sys::{constants::*};

const PS_MOVE_VID: u16 = 0x054c;
const PS_MOVE_PID: u16 = 0x03d5; // PSMove ZCM1
const PSMOVE_BTADDR_GET_ZCM1_SIZE: usize = 16;
const PSMOVE_BTADDR_GET_ZCM2_SIZE: usize = 21;
const PSMOVE_BTADDR_GET_MAX_SIZE: usize = PSMOVE_BTADDR_GET_ZCM2_SIZE;

enum PSMoveRequestType {
    GetBTAddr = 0x04,
}

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

fn main() {
    // get computer's bluetooth radio MAC
    let bt_mac = find_bluetooth_mac();
    // find PS Move controller
    // send radio MAC to controller (pair)
    //
    find_ps_move_usb().unwrap();
    // list_bluetooth_devices().unwrap();
}

fn find_bluetooth_mac() -> io::Result<()> {
    Ok(())
}

fn list_bluetooth_devices() -> io::Result<()> {
    println!("Scanning Bluetooth...");
    let devices = bt::discover_devices()?;
    println!("Bluetooth Devices:");
    for (idx, device) in devices.iter().enumerate() {
        println!("{}: {}", idx, *device);
    }

    if devices.len() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No Bluetooth devices found.",
        ));
    }

    let device_idx = request_device_idx(devices.len())?;

    let socket = BtStream::connect(iter::once(&devices[device_idx]), bt::BtProtocol::RFCOMM)?;

    match socket.peer_addr() {
        Ok(name) => println!("Peername: {}.", name.to_string()),
        Err(err) => println!("An error occured while retrieving the peername: {:?}", err),
    }

    match socket.local_addr() {
        Ok(name) => println!("Socket name: {}", name.to_string()),
        Err(err) => println!("An error occured while retrieving the sockname: {:?}", err),
    }

    let mut buffer = vec![0; 1024];
    loop {
        match socket.recv(&mut buffer[..]) {
            Ok(len) => println!("Received {} bytes.", len),
            Err(err) => return Err(err),
        }
    }
}

fn request_device_idx(len: usize) -> io::Result<usize> {
    println!("Please specify the index of the Bluetooth device you want to connect to:");

    let mut buffer = String::new();
    loop {
        io::stdin().read_line(&mut buffer)?;
        if let Ok(idx) = buffer.trim_end().parse::<usize>() {
            if idx < len {
                return Ok(idx);
            }
        }
        buffer.clear();
        println!("Invalid index. Please try again.");
    }
}

fn find_ps_move_usb() -> Result<()> {
    match rusb::Context::new() {
        Ok(mut context) => match open_device(&mut context, PS_MOVE_VID, PS_MOVE_PID) {
            Some((mut device, device_desc, mut handle)) => {
                // read_device(&mut device, &device_desc, &mut handle)
                // let interface = device_desc.descriptor.interface;
                let mut data = vec![0u8; PSMOVE_BTADDR_GET_ZCM1_SIZE+1];

                for n in 0..device_desc.num_configurations() {
                    let config_desc = match device.config_descriptor(n) {
                        Ok(c) => {
                            println!("n: {:?}, config {:?}", n, c);
                            c
                        },
                        Err(_) => unreachable!(),
                    };
                    for interface in config_desc.interfaces() {
                        for interface_desc in interface.descriptors() {
                            println!("interface_desc: {:?}", interface_desc);
                        }
                    }

                }


                data[0] = PSMoveRequestType::GetBTAddr as u8;
                println!("make data buffer");
                hid_get_feature_report(&mut handle, &mut data)
            }
            None => panic!("could not find device {:04x}:{:04x}", PS_MOVE_VID, PS_MOVE_PID),
        },
        Err(e) => panic!("could not initialize libusb: {}", e),
    }

    // println!("Scanning USB...");
    // let context = rusb::Context::new().unwrap();
    // for device in context.devices().unwrap().iter() {
    //     let device_desc = device.device_descriptor().unwrap();
    //     if device_desc.vendor_id() == PS_MOVE_VID && device_desc.product_id() == PS_MOVE_PID {
    //         println!("Found! Bus {:03} Device {:03} ID {:08x}:{:04x} Serial: {:?}",
    //             device.bus_number(),
    //             device.address(),
    //             device_desc.vendor_id(),
    //             device_desc.product_id());
    //     }
    // }
    // Ok(())
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        get_windows_usb_device_detail().unwrap();

        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };
        println!("device: {:?}", device);
        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, device_desc, handle)),
                Err(_) => continue,
            }
        }
    }

    None
}

fn read_device<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    handle: &mut DeviceHandle<T>,
) -> Result<()> {
    handle.reset()?;

    let timeout = Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;

    // println!("Active configuration: {}", handle.active_configuration()?);
    // println!("Languages: {:?}", languages);
    println!("Descriptor: {:?}", device_desc);

    if languages.len() > 0 {
        let language = languages[0];

        println!(
            "Manufacturer: {:?}",
            handle
                .read_manufacturer_string(language, device_desc, timeout)
                .ok()
        );
        println!(
            "Product: {:?}",
            handle
                .read_product_string(language, device_desc, timeout)
                .ok()
        );
        println!(
            "Serial Number: {:?}",
            handle
                .read_serial_number_string(language, device_desc, timeout)
                .ok()
        );
        // println!(
        //     "Path: {:?}",
        //     device_desc.
        //         .read_serial_number_string(language, device_desc, timeout)
        //         .ok()
        // );
    }

    // match find_readable_endpoint(device, device_desc, TransferType::Interrupt) {
    //     Some(endpoint) => read_endpoint(handle, endpoint, TransferType::Interrupt),
    //     None => println!("No readable interrupt endpoint"),
    // }

    match find_readable_endpoint(device, device_desc, TransferType::Control) {
        Some(endpoint) => read_endpoint(handle, endpoint, TransferType::Control),
        None => println!("No readable control endpoint"),
    }

    Ok(())
}

fn find_readable_endpoint<T: UsbContext>(
    device: &mut Device<T>,
    device_desc: &DeviceDescriptor,
    transfer_type: TransferType,
) -> Option<Endpoint> {
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    if endpoint_desc.direction() == Direction::In
                        && endpoint_desc.transfer_type() == transfer_type
                    {
                        return Some(Endpoint {
                            config: config_desc.number(),
                            iface: interface_desc.interface_number(),
                            setting: interface_desc.setting_number(),
                            address: endpoint_desc.address(),
                        });
                    }
                }
            }
        }
    }

    None
}

fn read_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: Endpoint,
    transfer_type: TransferType,
) {
    println!("Reading from endpoint: {:?}", endpoint);

    let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(endpoint.iface).ok();
            true
        }
        _ => false,
    };

    println!(" - kernel driver? {}", has_kernel_driver);

    match configure_endpoint(handle, &endpoint) {
        Ok(_) => {
            let mut vec = Vec::<u8>::with_capacity(256);
            let buf =
                unsafe { slice::from_raw_parts_mut((&mut vec[..]).as_mut_ptr(), vec.capacity()) };

            let timeout = Duration::from_secs(1);

            match transfer_type {
                TransferType::Interrupt => {
                    match handle.read_interrupt(endpoint.address, buf, timeout) {
                        Ok(len) => {
                            unsafe { vec.set_len(len) };
                            println!(" - read: {:?}", vec);
                        }
                        Err(err) => println!("could not read from endpoint: {}", err),
                    }
                }
                TransferType::Bulk => match handle.read_bulk(endpoint.address, buf, timeout) {
                    Ok(len) => {
                        unsafe { vec.set_len(len) };
                        println!(" - read: {:?}", vec);
                    }
                    Err(err) => println!("could not read from endpoint: {}", err),
                },
                _ => (),
            }
        }
        Err(err) => println!("could not configure endpoint: {}", err),
    }

    if has_kernel_driver {
        handle.attach_kernel_driver(endpoint.iface).ok();
    }
}

fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> Result<()> {
    handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)?;
    Ok(())
}

fn hid_get_feature_report<T: UsbContext>(handle: &mut DeviceHandle<T>, data: &mut [u8]) -> Result<()> {
    let timeout = Duration::from_secs(10);
    let report_number = data[0];

    println!("send data buffer: {:?}", data);
    match handle.read_control(
        LIBUSB_REQUEST_TYPE_CLASS|LIBUSB_RECIPIENT_INTERFACE|LIBUSB_ENDPOINT_IN,
        0x01, // HID get_report
        u16::from(0x03u8) << 8 | u16::from(report_number),
        0, // TODO: find this in handle
        data,
        timeout) {
            Ok(_size) => Ok(()),
            Err(e) => {
                println!("oh no! {:?}", data);
                Err(e)
            }
        }
}

use winapi::um::setupapi::{
    SP_DEVICE_INTERFACE_DATA,
    DIGCF_PRESENT, DIGCF_DEVICEINTERFACE,
    SetupDiGetDeviceInterfaceDetailA,
    SetupDiGetClassDevsA,
};

use winapi::shared::usbiodef::{
    GUID_DEVINTERFACE_USB_DEVICE
};

use std::ptr;

fn get_windows_usb_device_detail() -> Result<()> {
    // SP_DEVICE_INTERFACE_DATA
    let device_info_set = unsafe { SetupDiGetClassDevsA(&GUID_DEVINTERFACE_USB_DEVICE, ptr::null(), ptr::null_mut(), DIGCF_PRESENT | DIGCF_DEVICEINTERFACE) };
    println!("device_info: {:?}", device_info_set);
    Ok(())
}