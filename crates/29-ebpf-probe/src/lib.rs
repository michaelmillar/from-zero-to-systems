// ============================================================
//  YOUR CHALLENGE - implement an eBPF userspace loader with aya.
//
//  eBPF programs run inside the Linux kernel in a sandboxed VM.
//  They are attached to kernel hooks (kprobes, tracepoints, XDP)
//  and communicate with userspace via BPF maps - shared key-value
//  stores accessible from both kernel and userspace Rust code.
//
//  Your job is the userspace side:
//    1. Load the pre-compiled BPF ELF object from bpf/*.bpf.o
//    2. Attach the programs to the correct kernel hooks
//    3. Read events and counters from BPF maps
//
//  Prerequisites: 28-raw-socket (packet parsing intuition),
//  Linux >= 5.15 with BTF enabled (/sys/kernel/btf/vmlinux),
//  CAP_BPF capability (run tests with sudo).
//
//  See bpf/README.md for how to rebuild the BPF ELF objects.
// ============================================================

use std::collections::HashMap;

/// An exec event recorded by the kprobe on sys_execve.
#[derive(Debug, Clone)]
pub struct ExecEvent {
    pub pid: u32,
    pub comm: String,
}

/// Load the pre-compiled execve kprobe ELF and attach it to sys_execve.
///
/// The ELF bytes are embedded at compile time:
///   let elf = include_bytes!("../bpf/execve_probe.bpf.o");
///
/// Steps:
///   1. aya::Bpf::load(elf)?
///   2. bpf.program_mut("execve_probe") - cast to KProbe
///   3. prog.load()?
///   4. prog.attach("sys_execve", 0)?
///   5. Return the Bpf handle (caller reads maps from it)
///
/// Requires: Linux >= 5.15, CAP_BPF
#[cfg(target_os = "linux")]
pub fn load_execve_probe() -> Result<aya::Bpf, aya::BpfError> {
    todo!()
}

/// Read all exec events from the "exec_events" HashMap BPF map.
///
/// Map layout: key = pid (u32), value = comm ([u8; 16], null-terminated).
///
/// Steps:
///   1. bpf.map("exec_events") - cast to aya::maps::HashMap<_, u32, [u8; 16]>
///   2. Iterate, convert comm bytes to String (trim trailing null bytes)
#[cfg(target_os = "linux")]
pub fn read_exec_events(bpf: &aya::Bpf) -> Result<Vec<ExecEvent>, aya::maps::MapError> {
    todo!()
}

/// Load the XDP packet counter ELF and attach it to `iface`.
///
/// Steps:
///   1. aya::Bpf::load(include_bytes!("../bpf/xdp_counter.bpf.o"))?
///   2. Get program "xdp_counter", cast to Xdp, load()
///   3. prog.attach(iface, XdpFlags::default())?
///   4. Return the Bpf handle
///
/// Requires: Linux >= 5.15, CAP_NET_ADMIN
#[cfg(target_os = "linux")]
pub fn load_xdp_counter(iface: &str) -> Result<aya::Bpf, aya::BpfError> {
    todo!()
}

/// Read packet counts from the "packet_counts" Array BPF map.
///
/// The array is indexed by IP protocol number (0-255); values are u64 packet counts.
/// Return only entries with count > 0 as HashMap<protocol_number, count>.
#[cfg(target_os = "linux")]
pub fn read_packet_counts(bpf: &aya::Bpf) -> Result<HashMap<u8, u64>, aya::maps::MapError> {
    todo!()
}

// ============================================================
//  UNIT TESTS (no kernel required - run anywhere)
// ============================================================

#[cfg(test)]
mod tests {
    mod protocol_classification {
        /// Given an IP protocol number, return a human-readable label.
        pub fn classify_protocol(proto: u8) -> &'static str {
            match proto {
                1  => "ICMP",
                6  => "TCP",
                17 => "UDP",
                _  => "other",
            }
        }

        #[test]
        fn protocol_classifier_identifies_tcp() {
            assert_eq!(classify_protocol(6), "TCP");
        }

        #[test]
        fn protocol_classifier_identifies_udp() {
            assert_eq!(classify_protocol(17), "UDP");
        }

        #[test]
        fn protocol_classifier_identifies_icmp() {
            assert_eq!(classify_protocol(1), "ICMP");
        }

        #[test]
        fn unknown_protocol_returns_other() {
            assert_eq!(classify_protocol(255), "other");
        }
    }

    mod event_serialisation {
        use super::super::ExecEvent;

        #[test]
        fn exec_event_deserialises_correctly() {
            // Simulate a comm value read from a BPF map: null-terminated 16-byte array
            let mut comm_bytes = [0u8; 16];
            comm_bytes[..4].copy_from_slice(b"bash");
            let comm = String::from_utf8_lossy(&comm_bytes)
                .trim_end_matches('\0')
                .to_string();
            let event = ExecEvent { pid: 1234, comm };
            assert_eq!(event.pid, 1234);
            assert_eq!(event.comm, "bash");
        }

        #[test]
        fn map_key_round_trips_correctly() {
            // BPF maps on x86 are little-endian; verify u32 PID round-trips
            let pid: u32 = 0xDEAD_BEEF;
            let bytes = pid.to_le_bytes();
            let recovered = u32::from_le_bytes(bytes);
            assert_eq!(recovered, pid);
        }
    }
}
