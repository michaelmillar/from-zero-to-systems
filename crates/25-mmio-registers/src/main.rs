fn main() {
    use mmio_registers::{Permission, RegisterBlock};

    let mut rb = RegisterBlock::new(8);
    rb.set_permission(0, Permission::ReadOnly);  // status register
    rb.set_permission(1, Permission::WriteOnly); // command register

    // Simulate a hardware status register: bits 0-3 = error code, bits 4-7 = state
    rb.write32(2, 0b0001_0011).unwrap();
    let error_code = rb.read_field(2, 0, 4).unwrap();
    let state      = rb.read_field(2, 4, 4).unwrap();
    println!("peripheral state={state} error_code={error_code}");
}
