// ============================================================
//  YOUR CHALLENGE - implement a character device driver.
//
//  Linux exposes hardware as files under /dev. Every character
//  device must implement a fixed set of operations: open, read,
//  write, and ioctl. The kernel dispatches to the right driver
//  via a function pointer table known as file_operations (fops).
//
//  Here you implement that dispatch table as a Rust trait and
//  build a concrete SimpleDevice that simulates the hardware
//  from 25-mmio-registers using an internal byte buffer.
//
//  Think of SimpleDevice as a driver for a FIFO peripheral:
//  writes push bytes in, reads pop bytes out.
// ============================================================

#[derive(Debug, PartialEq)]
pub enum DeviceError {
    /// Device is already held by another caller.
    Busy,
    /// Operation requires the device to be open first.
    NotOpen,
    /// ioctl command code is not recognised.
    InvalidCommand,
    /// Generic I/O failure.
    Io(String),
}

/// The ioctl command codes for SimpleDevice.
pub mod cmd {
    /// Reset: discard all buffered data, arg is ignored.
    pub const RESET: u32 = 0x01;
    /// SetMode: store arg as the current operating mode.
    pub const SET_MODE: u32 = 0x02;
    /// GetStatus: return 1 if there is buffered data, 0 otherwise.
    pub const GET_STATUS: u32 = 0x03;
}

/// The character device interface.
/// Mirrors Linux's struct file_operations for the four core operations.
pub trait CharDevice {
    /// Claim exclusive access to the device.
    /// Returns Err(Busy) if already open.
    fn open(&mut self) -> Result<(), DeviceError>;

    /// Release exclusive access.
    fn close(&mut self);

    /// Copy up to buf.len() bytes from device into buf.
    /// Returns the number of bytes actually copied.
    /// Returns Err(NotOpen) if the device has not been opened.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DeviceError>;

    /// Write all bytes from buf into the device.
    /// Returns the number of bytes accepted.
    /// Returns Err(NotOpen) if the device has not been opened.
    fn write(&mut self, buf: &[u8]) -> Result<usize, DeviceError>;

    /// Send a control command.
    ///
    /// cmd: one of the constants in the `cmd` module.
    /// arg: optional argument (semantics depend on cmd).
    /// Returns a 64-bit response value, or Err(InvalidCommand) for unknown codes.
    fn ioctl(&mut self, cmd: u32, arg: u64) -> Result<u64, DeviceError>;
}

/// A FIFO character device backed by an in-memory byte buffer.
/// Simulates a hardware peripheral's data register.
pub struct SimpleDevice {
    is_open: bool,
    buffer: Vec<u8>,
    mode: u64,
}

impl SimpleDevice {
    /// Create a new, closed device.
    pub fn new() -> Self {
        todo!()
    }
}

impl Default for SimpleDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl CharDevice for SimpleDevice {
    /// Claim the device.
    /// Returns Err(Busy) if already open.
    fn open(&mut self) -> Result<(), DeviceError> {
        todo!()
    }

    /// Release the device. A closed device can be re-opened.
    fn close(&mut self) {
        todo!()
    }

    /// Drain up to buf.len() bytes from the front of the internal buffer.
    /// Returns the number of bytes copied (may be less than buf.len()).
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DeviceError> {
        todo!()
    }

    /// Append all bytes from buf to the internal buffer.
    fn write(&mut self, buf: &[u8]) -> Result<usize, DeviceError> {
        todo!()
    }

    /// Dispatch ioctl commands:
    ///   cmd::RESET      - clear internal buffer, return 0
    ///   cmd::SET_MODE   - store arg as mode, return 0
    ///   cmd::GET_STATUS - return 1 if buffer non-empty, 0 if empty
    ///   anything else   - Err(InvalidCommand)
    fn ioctl(&mut self, cmd: u32, arg: u64) -> Result<u64, DeviceError> {
        todo!()
    }
}

// ============================================================
//  TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::cmd;

    mod opening_and_closing {
        use super::*;

        #[test]
        fn open_succeeds_on_fresh_device() {
            let mut dev = SimpleDevice::new();
            assert!(dev.open().is_ok());
        }

        #[test]
        fn second_open_returns_busy() {
            let mut dev = SimpleDevice::new();
            dev.open().unwrap();
            assert_eq!(dev.open(), Err(DeviceError::Busy));
        }

        #[test]
        fn close_allows_reopen() {
            let mut dev = SimpleDevice::new();
            dev.open().unwrap();
            dev.close();
            assert!(dev.open().is_ok());
        }
    }

    mod reading_and_writing {
        use super::*;

        #[test]
        fn write_then_read_returns_same_bytes() {
            let mut dev = SimpleDevice::new();
            dev.open().unwrap();
            dev.write(b"hello").unwrap();
            let mut buf = vec![0u8; 5];
            let n = dev.read(&mut buf).unwrap();
            assert_eq!(n, 5);
            assert_eq!(&buf[..n], b"hello");
        }

        #[test]
        fn read_on_closed_device_returns_not_open() {
            let mut dev = SimpleDevice::new();
            let mut buf = [0u8; 4];
            assert_eq!(dev.read(&mut buf), Err(DeviceError::NotOpen));
        }

        #[test]
        fn write_on_closed_device_returns_not_open() {
            let mut dev = SimpleDevice::new();
            assert_eq!(dev.write(b"data"), Err(DeviceError::NotOpen));
        }

        #[test]
        fn partial_read_drains_only_requested_bytes() {
            let mut dev = SimpleDevice::new();
            dev.open().unwrap();
            dev.write(b"abcdef").unwrap();
            let mut buf = [0u8; 3];
            let n = dev.read(&mut buf).unwrap();
            assert_eq!(n, 3);
            assert_eq!(&buf, b"abc");
            let mut rest = [0u8; 8];
            let m = dev.read(&mut rest).unwrap();
            assert_eq!(m, 3);
            assert_eq!(&rest[..m], b"def");
        }
    }

    mod ioctl_dispatch {
        use super::*;

        #[test]
        fn reset_clears_internal_buffer() {
            let mut dev = SimpleDevice::new();
            dev.open().unwrap();
            dev.write(b"data").unwrap();
            dev.ioctl(cmd::RESET, 0).unwrap();
            let mut buf = [0u8; 8];
            assert_eq!(dev.read(&mut buf).unwrap(), 0);
        }

        #[test]
        fn set_mode_stores_mode_value() {
            let mut dev = SimpleDevice::new();
            dev.open().unwrap();
            dev.ioctl(cmd::SET_MODE, 42).unwrap();
            assert_eq!(dev.ioctl(cmd::GET_STATUS, 0).unwrap(), 0);
        }

        #[test]
        fn get_status_returns_nonzero_when_data_buffered() {
            let mut dev = SimpleDevice::new();
            dev.open().unwrap();
            dev.write(b"x").unwrap();
            assert_eq!(dev.ioctl(cmd::GET_STATUS, 0).unwrap(), 1);
        }

        #[test]
        fn unknown_command_returns_invalid_command() {
            let mut dev = SimpleDevice::new();
            dev.open().unwrap();
            assert_eq!(dev.ioctl(0xFF, 0), Err(DeviceError::InvalidCommand));
        }
    }
}
