fn main() {
    use char_device_driver::{CharDevice, SimpleDevice, cmd};

    let mut dev = SimpleDevice::new();
    dev.open().expect("open failed");

    dev.write(b"sensor_reading: 42.7\n").unwrap();
    let status = dev.ioctl(cmd::GET_STATUS, 0).unwrap();
    println!("buffer has data: {status}");

    let mut out = vec![0u8; 64];
    let n = dev.read(&mut out).unwrap();
    println!("{}", String::from_utf8_lossy(&out[..n]));
    dev.close();
}
