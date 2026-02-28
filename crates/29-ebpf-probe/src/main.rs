fn main() {
    println!("ebpf-probe: requires Linux >= 5.15 with BTF and CAP_BPF.");
    println!("Unit tests run anywhere: cargo test -p ebpf-probe");
    println!("Integration tests: sudo cargo test -p ebpf-probe");
    println!("See bpf/README.md for build instructions.");
}
