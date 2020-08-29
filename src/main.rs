use rusb::{Context, TransferType};
use fingerprint_driver::{open_device, find_readable_endpoint, get_fingerprint, save_fingerprint};

const VENDOR_ID: u16 = 0x1c7a;
const PRODUCT_ID: u16 = 0x0570;
const FINGERPRINT_FILENAME: &str = "fingerprint";

fn main() {
    match Context::new() {
        Ok(mut context) => match open_device(&mut context, VENDOR_ID, PRODUCT_ID) {
            Some((device, device_desc, mut handle)) => {
                match find_readable_endpoint(device, &device_desc, TransferType::Bulk) {
                    Some(endpoint) => {
                        let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
                            Ok(true) => {
                                handle.detach_kernel_driver(endpoint.iface).ok();
                                true
                            }
                            _ => false,
                        };

                        match get_fingerprint(&mut handle, &endpoint) {
                            Some(finger_print) => {
                                save_fingerprint(finger_print, FINGERPRINT_FILENAME);
                                println!("Fingerprint saved");
                            },
                            None => println!("Couldn't retrieve data from device")
                        }


                        if has_kernel_driver {
                            handle.attach_kernel_driver(endpoint.iface).ok();
                        }
                    }
                    None => println!("No readable bulk endpoint"),
                }

                //view_device_info(&device_desc, &mut handle);
            }
            None => println!("could not find device {}:{}", VENDOR_ID, PRODUCT_ID),
        },
        Err(e) => panic!("could not initialize libusb: {}", e),
    };
}
