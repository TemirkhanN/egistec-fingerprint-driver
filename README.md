# EgisTec Touch Fingerprint Sensor driver
Rust sandbox implementing fingerprint sensor driver.  
Based on [saeedark/egis0570](https://github.com/saeedark/egis0570).  
This is not driver yet!  
In current state reader gets signal and returns scan that is saved into `fingerprint.pgm`.  

## Test
Execution means access to fingerprint sensor. That requires superuser permissions.
```bash
cargo build
sudo ./target/debug/fingerprint-driver
```
fingerprint.pgm will be created at execution triggered directory(basically at project root).  

Tested on `acer swift sf313-52g`.
Manufacturer: EgisTec
Product: EgisTec Touch Fingerprint Sensor
Serial Number: 0E1BDD3B