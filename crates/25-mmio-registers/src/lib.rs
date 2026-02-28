// ============================================================
//  YOUR CHALLENGE - implement a memory-mapped register block.
//
//  Hardware peripherals expose their controls as fixed memory
//  addresses. The CPU reads and writes those addresses with
//  "volatile" semantics, preventing the compiler from caching
//  values across reads (since hardware can change them at any time).
//
//  Here you simulate that with a Vec<u32> backing buffer and
//  raw pointer volatile reads/writes.
//
//  The bitfield helpers (read_field, write_field) should use the
//  same masking and shifting techniques as 09-bit-manipulator.
//
//  Hint for volatile: use std::ptr::read_volatile / write_volatile
//  on a raw pointer into self.buffer.
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Permission {
    ReadWrite,
    ReadOnly,
    WriteOnly,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegError {
    OutOfBounds,
    PermissionDenied,
}

pub struct RegisterBlock {
    buffer: Vec<u32>,
    perms: Vec<Permission>,
}

impl RegisterBlock {
    /// Create a new register block with `count` 32-bit registers, all ReadWrite.
    pub fn new(count: usize) -> Self {
        Self {
            buffer: vec![0u32; count],
            perms: vec![Permission::ReadWrite; count],
        }
    }

    /// Set the access permission for register at `index`.
    pub fn set_permission(&mut self, index: usize, perm: Permission) {
        if index < self.perms.len() {
            self.perms[index] = perm;
        }
    }

    /// Read the 32-bit value at `index` using volatile semantics.
    ///
    /// Returns Err(OutOfBounds) if index >= register count.
    /// Returns Err(PermissionDenied) if the register is WriteOnly.
    ///
    /// Implementation: check bounds, check permission, then:
    ///   unsafe { std::ptr::read_volatile(self.buffer.as_ptr().add(index)) }
    pub fn read32(&self, index: usize) -> Result<u32, RegError> {
        todo!()
    }

    /// Write `value` to the register at `index` using volatile semantics.
    ///
    /// Returns Err(OutOfBounds) if index >= register count.
    /// Returns Err(PermissionDenied) if the register is ReadOnly.
    ///
    /// Implementation: check bounds, check permission, then:
    ///   unsafe { std::ptr::write_volatile(self.buffer.as_mut_ptr().add(index), value) }
    pub fn write32(&mut self, index: usize, value: u32) -> Result<(), RegError> {
        todo!()
    }

    /// Extract `len` bits starting at `bit_offset` (0 = LSB) from register `index`.
    ///
    /// Hint: read32(index)?, then apply a mask: (value >> bit_offset) & ((1 << len) - 1)
    /// This is identical to the extract_bits logic in 09-bit-manipulator.
    pub fn read_field(&self, index: usize, bit_offset: u8, len: u8) -> Result<u32, RegError> {
        todo!()
    }

    /// Write `value` into bits [bit_offset, bit_offset+len) of register `index`
    /// without modifying any other bits.
    ///
    /// Algorithm:
    ///   1. Build a mask: ((1u32 << len) - 1) << bit_offset
    ///   2. Read current register value (read32)
    ///   3. Clear the target field:  current & !mask
    ///   4. Insert new bits:          cleared | ((value << bit_offset) & mask)
    ///   5. Write back (write32)
    pub fn write_field(&mut self, index: usize, bit_offset: u8, len: u8, value: u32) -> Result<(), RegError> {
        todo!()
    }
}

// ============================================================
//  TESTS - these ARE the specification.
//  Run `cargo test -p mmio-registers` to see them fail.
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod read_write {
        use super::*;

        #[test]
        fn write_then_read_returns_same_value() {
            let mut rb = RegisterBlock::new(4);
            rb.write32(0, 0xDEAD_BEEF).unwrap();
            assert_eq!(rb.read32(0).unwrap(), 0xDEAD_BEEF);
        }

        #[test]
        fn write_to_one_register_does_not_affect_adjacent_register() {
            let mut rb = RegisterBlock::new(4);
            rb.write32(0, 0xFF).unwrap();
            assert_eq!(rb.read32(1).unwrap(), 0);
        }

        #[test]
        fn out_of_bounds_read_returns_err() {
            let rb = RegisterBlock::new(4);
            assert_eq!(rb.read32(4), Err(RegError::OutOfBounds));
        }

        #[test]
        fn out_of_bounds_write_returns_err() {
            let mut rb = RegisterBlock::new(4);
            assert_eq!(rb.write32(4, 0), Err(RegError::OutOfBounds));
        }
    }

    mod permissions {
        use super::*;

        #[test]
        fn write_to_readonly_register_returns_permission_denied() {
            let mut rb = RegisterBlock::new(4);
            rb.set_permission(0, Permission::ReadOnly);
            assert_eq!(rb.write32(0, 42), Err(RegError::PermissionDenied));
        }

        #[test]
        fn read_from_writeonly_register_returns_permission_denied() {
            let mut rb = RegisterBlock::new(4);
            rb.set_permission(1, Permission::WriteOnly);
            assert_eq!(rb.read32(1), Err(RegError::PermissionDenied));
        }

        #[test]
        fn readonly_register_can_be_read() {
            let mut rb = RegisterBlock::new(4);
            rb.set_permission(2, Permission::ReadOnly);
            assert!(rb.read32(2).is_ok());
        }
    }

    mod bitfields {
        use super::*;

        #[test]
        fn read_field_extracts_correct_bits() {
            let mut rb = RegisterBlock::new(1);
            // 0b1011_0110: bits 2..=5 are 0b1101
            rb.write32(0, 0b1011_0110).unwrap();
            assert_eq!(rb.read_field(0, 2, 4).unwrap(), 0b1101);
        }

        #[test]
        fn write_field_does_not_disturb_adjacent_bits() {
            let mut rb = RegisterBlock::new(1);
            rb.write32(0, 0xFFFF_FFFF).unwrap();
            // Zero out bits 4..=7 (4 bits starting at offset 4)
            rb.write_field(0, 4, 4, 0).unwrap();
            // Bits 4..=7 are now 0, everything else still 1
            assert_eq!(rb.read32(0).unwrap(), 0xFFFF_FF0F);
        }

        #[test]
        fn write_field_sets_target_bits_correctly() {
            let mut rb = RegisterBlock::new(1);
            rb.write32(0, 0).unwrap();
            rb.write_field(0, 0, 4, 0b1010).unwrap();
            assert_eq!(rb.read32(0).unwrap(), 0b1010);
        }
    }
}
