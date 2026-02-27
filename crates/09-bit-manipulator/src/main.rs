use bit_manipulator::*;

fn main() {
    println!("=== Bit Manipulation: IPv4 Packet Inspection ===\n");

    // Simulate a raw IPv4 header (20 bytes minimum)
    // Version=4, IHL=5, DSCP=0, Total Length=60
    // TTL=64, Protocol=6 (TCP)
    // Src: 192.168.1.100,  Dst: 93.184.216.34 (example.com)
    let header: [u8; 20] = [
        0x45, 0x00, 0x00, 0x3C,  // Ver/IHL, DSCP, Total Length
        0x1A, 0x2B, 0x40, 0x00,  // ID, Flags/Fragment Offset
        0x40, 0x06, 0x00, 0x00,  // TTL=64, Protocol=TCP(6), Checksum
        192, 168, 1, 100,         // Source IP
        93, 184, 216, 34,         // Destination IP
    ];

    let byte0 = header[0];
    let (s0, s1, s2, s3) = ipv4_src_addr(&header);

    println!("  Header byte 0:    0x{byte0:02X} = 0b{byte0:08b}");
    println!("  IP Version:       {}", ipv4_version(byte0));
    println!("  IHL:              {} ({} bytes)", ipv4_ihl(byte0), ipv4_ihl(byte0) * 4);
    println!("  TTL:              {}", header[8]);
    println!("  Protocol:         {} (6=TCP)", header[9]);
    println!("  Source IP:        {s0}.{s1}.{s2}.{s3}");

    println!("\n=== Bit Tricks ===\n");

    let val: u32 = 0b10110110_11001010_11110000_00001111;
    println!("  Value:            0x{val:08X} = 0b{val:032b}");
    println!("  count_ones:       {} set bits", count_ones(val));
    println!("  parity:           {} (odd set bits = true)", parity(val));
    println!("  swap_bytes:       0x{:08X}", swap_bytes(val));
    println!("  rotate_left(8):   0x{:08X}", rotate_left(val, 8));
    println!("  extract [8..16]:  0b{:08b}", extract_bits(val, 8, 8));
}
