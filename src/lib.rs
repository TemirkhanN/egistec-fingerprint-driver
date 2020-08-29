use std::{time::Duration};
use rusb::{
    Device, DeviceDescriptor, DeviceHandle, Direction, Result, TransferType, UsbContext,
};
use std::io::Write;
use std::fs::{File, remove_file};
use std::path::Path;

#[derive(Debug)]
pub struct Endpoint {
    config: u8,
    pub iface: u8,
    setting: u8,
    address: u8,
}

const DEVICE_SIGNALS: [[u8; 7]; 23] = [
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x20, 0x3f],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x58, 0x3f],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x21, 0x09],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x57, 0x09],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x22, 0x03],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x56, 0x03],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x23, 0x01],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x55, 0x01],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x24, 0x01],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x54, 0x01],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x16, 0x3e],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x09, 0x0b],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x14, 0x03],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x15, 0x00],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x02, 0x0f],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x10, 0x00],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x11, 0x38],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x12, 0x00],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x13, 0x71],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x03, 0x80],
    [0x45, 0x47, 0x49, 0x53, 0x00, 0x02, 0x80],
    [0x45, 0x47, 0x49, 0x53, 0x01, 0x02, 0x2f],
    [0x45, 0x47, 0x49, 0x53, 0x06, 0x00, 0xfe]
];

const LIBUSB_OUTPUT_ADDRESS: u8 = 0x04;

pub fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, device_desc, handle)),
                Err(e) => {
                    println!("Couldnt access device: {}", e);
                }
            }
        }
    }

    None
}

pub fn find_readable_endpoint<T: UsbContext>(
    device: Device<T>,
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

pub fn get_fingerprint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> Option<[u8; 32512]> {
    println!("Reading from endpoint: {:?}", endpoint);

    match configure_endpoint(handle, &endpoint) {
        Ok(_) => {
            let timeout = Duration::from_secs(1);
            let mut finger_print: [u8; 32512] = [0; 32512];
            for signal in DEVICE_SIGNALS.iter() {
                match handle.write_bulk(LIBUSB_OUTPUT_ADDRESS, signal, timeout) {
                    Ok(_len) => {
                        //println!(" - write: {:?}", &signal[.._len]);
                    }
                    Err(err) => println!("could not communicate with device: {}", err),
                }
                match handle.read_bulk(endpoint.address, &mut finger_print, timeout) {
                    Ok(_len) => {
                        //println!("inputbuff {:?}", &finger_print[..len]);
                    }
                    Err(err) => println!("could not read from endpoint: {}", err),
                }
            }

            return Some(finger_print);
        }
        Err(err) => println!("could not configure endpoint: {}", err),
    }

    return None;
}

pub fn save_fingerprint(binary_data: [u8; 32512], file_name: &str) {
    let file_path = format!("{}.pgm", file_name);

    if Path::new(&file_path).exists() {
        delete_file(&file_path);
    }

    let mut file = File::create(&file_path).unwrap();
    let img_width = 115;
    let img_height = 284;

    match file.write(format!("P5\n{} {}\n{}\n", img_width, img_height / 5, 255).as_bytes()) {
        Err(e) => {
            println!("Couldn't write to fingerprint file: {}", e);

            return;
        }
        _ => {}
    }

    let mut i = 0;
    let mut file_content: Vec<u8> = Vec::new();
    // yep its X in height. don't mind
    for _x in 0..img_height {
        for _y in 0..img_width - 1 {
            i += 1;
            file_content.push(binary_data[i]);
        }
        file_content.push(0xa); // line feed
    }

    match file.write(&file_content.into_boxed_slice()) {
        Err(e) => {
            println!("Couldn't write to fingerprint file: {}", e);
            delete_file(&file_path);
        }
        _ => {}
    }
}

fn delete_file(file_path: &String) {
    match remove_file(file_path) {
        Err(e) => {
            println!("Couldn't delete existing fingerprint: {}", e);

            return;
        }
        _ => {}
    };
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

/*
pub fn view_device_info<T: UsbContext>(
    device_desc: &DeviceDescriptor,
    handle: &mut DeviceHandle<T>,
) -> Result<()> {
    handle.reset()?;

    let timeout = Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;

    println!("Active configuration: {}", handle.active_configuration()?);
    println!("Languages: {:?}", languages);

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
    }

    Ok(())
}
*/