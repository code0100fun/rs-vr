use winapi::um::bluetoothapis::{
    BLUETOOTH_FIND_RADIO_PARAMS,
    BLUETOOTH_RADIO_INFO,
    BLUETOOTH_ADDRESS,
    BluetoothFindFirstRadio,
    BluetoothFindRadioClose,
    BluetoothGetRadioInfo,
};
use winapi::um::winnt::{
    HANDLE,
};
use winapi::um::handleapi::{
    INVALID_HANDLE_VALUE,
};

use io_bluetooth::bt::{self, BtStream};
use std::io;
use std::iter;

pub fn get_host_address() -> io::Result<String> {
    let bt_handle = find_first_bluetooth_radio()?;
    let radio_info = get_radio_info(bt_handle)?;
    Ok(bluetooth_address_to_string(radio_info.address))
}

fn get_radio_info(bt_handle: HANDLE) -> io::Result<BLUETOOTH_RADIO_INFO> {
    let mut radio_info = BLUETOOTH_RADIO_INFO::default();
    radio_info.dwSize = std::mem::size_of::<BLUETOOTH_RADIO_INFO>() as u32;
    unsafe {
        BluetoothGetRadioInfo(bt_handle, &mut radio_info);
    }
    Ok(radio_info)
}

fn find_first_bluetooth_radio() -> io::Result<HANDLE> {
    let radio_params = BLUETOOTH_FIND_RADIO_PARAMS {
        dwSize: std::mem::size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32
    };
    let mut handle: HANDLE = INVALID_HANDLE_VALUE;
    let h_find = unsafe { BluetoothFindFirstRadio(&radio_params, &mut handle) };

    if h_find != std::ptr::null_mut() {
        unsafe { BluetoothFindRadioClose(h_find) };
    }
    Ok(handle)
}

fn bluetooth_address_to_string(bt_address: BLUETOOTH_ADDRESS) -> String {
    let addr = format!("{:012x}", bt_address);
    let pairs: Vec<&[u8]> = addr.as_bytes().chunks(2).collect();
    let pairs = pairs.join(&(':' as u8));
    String::from_utf8(pairs).unwrap()
}

pub fn list_bluetooth_devices() -> io::Result<()> {
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