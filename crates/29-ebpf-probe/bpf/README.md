# Pre-compiled BPF objects

The `.bpf.o` files in this directory are pre-compiled eBPF programs loaded
at runtime by the Rust userspace loader in `src/lib.rs`.

## Programs

- `execve_probe.bpf.o` — kprobe on `sys_execve`, writes `ExecEvent` into a `HashMap` BPF map
- `xdp_counter.bpf.o`  — XDP program, counts packets by IP protocol into an `Array` map

## Rebuilding

Requires `bpf-linker` and Rust nightly targeting `bpfel-unknown-none`:

```bash
rustup target add bpfel-unknown-none
cargo install bpf-linker
```

See [aya-rs/aya-template](https://github.com/aya-rs/aya-template) for the full two-crate
build pattern used in production eBPF projects.

## Runtime requirements

- Linux kernel >= 5.15
- BTF enabled: `/sys/kernel/btf/vmlinux` must exist
- `CAP_BPF + CAP_PERFMON` for kprobes; `CAP_NET_ADMIN` for XDP
- Check: `sudo bpftool feature probe kernel | grep bpf_syscall`
