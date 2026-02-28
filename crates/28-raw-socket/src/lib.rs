// ============================================================
//  YOUR CHALLENGE - parse raw network packets.
//
//  Packet parsing means reading wire-format byte buffers
//  directly, without a parsing library. You use byte offsets
//  and the bit-manipulation techniques from 09-bit-manipulator.
//
//  The three protocols here are stacked (Ethernet wraps IP, IP
//  wraps TCP) - the same nesting model used in every network
//  stack from Linux to P4 programmable hardware.
//
//  ConnTracker uses a HashSet of 4-tuples to detect repeated
//  connections. In production (see 14-bloom-filter), this would
//  be a probabilistic structure to handle millions of flows.
//
//  Packet layout reference:
//    Ethernet: dst(6) src(6) ethertype(2) payload(...)
//    IPv4:     version/ihl(1) dscp(1) total_len(2) id(2)
//              flags/frag(2) ttl(1) proto(1) cksum(2)
//              src_ip(4) dst_ip(4) [options] payload
//    TCP:      src_port(2) dst_port(2) seq(4) ack(4)
//              data_offset/flags(2) window(2) cksum(2) urgent(2)
// ============================================================

use std::collections::HashSet;

pub const ETHERTYPE_IPV4: u16 = 0x0800;
pub const ETHERTYPE_ARP: u16  = 0x0806;

pub const PROTO_TCP: u8  = 6;
pub const PROTO_UDP: u8  = 17;
pub const PROTO_ICMP: u8 = 1;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    TooShort,
    UnknownEthertype(u16),
    InvalidChecksum,
    UnknownProtocol(u8),
}

#[derive(Debug, PartialEq, Clone)]
pub struct EthernetFrame<'a> {
    pub dst: [u8; 6],
    pub src: [u8; 6],
    pub ethertype: u16,
    pub payload: &'a [u8],
}

#[derive(Debug, PartialEq, Clone)]
pub struct Ipv4Packet<'a> {
    pub src: [u8; 4],
    pub dst: [u8; 4],
    pub protocol: u8,
    pub ttl: u8,
    pub payload: &'a [u8],
}

#[derive(Debug, PartialEq, Clone)]
pub struct TcpSegment {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq: u32,
    /// Lower byte of the TCP flags field (SYN=0x02, ACK=0x10, FIN=0x01, RST=0x04).
    pub flags: u8,
}

/// Parse an Ethernet frame from raw bytes.
///
/// Layout: [dst: 6 bytes][src: 6 bytes][ethertype: 2 bytes, big-endian][payload: rest]
/// Minimum frame size is 14 bytes. Return Err(TooShort) if shorter.
/// Return Err(UnknownEthertype) for any ethertype other than IPv4 or ARP.
pub fn parse_ethernet(buf: &[u8]) -> Result<EthernetFrame<'_>, ParseError> {
    todo!()
}

/// Parse an IPv4 packet from raw bytes (the payload of an Ethernet frame).
///
/// Byte 0:    version (upper 4 bits) and IHL (lower 4 bits; IHL * 4 = header bytes)
/// Byte 8:    TTL
/// Byte 9:    Protocol
/// Bytes 10-11: header checksum - validate with ipv4_checksum()
/// Bytes 12-15: source IP
/// Bytes 16-19: destination IP
/// Payload starts at byte (IHL * 4).
///
/// Return Err(TooShort) if buf is shorter than IHL * 4.
/// Return Err(InvalidChecksum) if the header checksum does not validate.
pub fn parse_ipv4(buf: &[u8]) -> Result<Ipv4Packet<'_>, ParseError> {
    todo!()
}

/// Parse a TCP segment from raw bytes (the payload of an IPv4 packet).
///
/// Bytes 0-1:  source port (big-endian u16)
/// Bytes 2-3:  destination port (big-endian u16)
/// Bytes 4-7:  sequence number (big-endian u32)
/// Byte 13:    flags byte (SYN=0x02, ACK=0x10, FIN=0x01, RST=0x04)
pub fn parse_tcp(buf: &[u8]) -> Result<TcpSegment, ParseError> {
    todo!()
}

/// Validate an IPv4 header checksum.
///
/// Algorithm: interpret each pair of bytes as a big-endian u16 and sum them as u32.
/// Fold carry: while sum >> 16 != 0 { sum = (sum & 0xFFFF) + (sum >> 16) }
/// One's complement the lower 16 bits: !sum as u16.
/// A valid header produces 0xFFFF after this operation.
///
/// Returns true if the checksum is valid.
pub fn ipv4_checksum(header: &[u8]) -> bool {
    todo!()
}

/// A stateful connection tracker.
///
/// Tracks (src_ip, dst_ip, src_port, dst_port) 4-tuples to identify
/// repeated connections. In production see 14-bloom-filter for a
/// probabilistic version that handles millions of flows.
pub struct ConnTracker {
    seen: HashSet<(u32, u32, u16, u16)>,
}

impl ConnTracker {
    pub fn new() -> Self {
        Self { seen: HashSet::new() }
    }

    /// Record a connection and return whether it was already seen.
    ///
    /// Encode IPs as u32 with u32::from_be_bytes(ip.src).
    /// Returns true if this 4-tuple was already in the tracker (duplicate).
    pub fn observe(&mut self, ip: &Ipv4Packet<'_>, tcp: &TcpSegment) -> bool {
        todo!()
    }

    /// Number of unique 4-tuples tracked.
    pub fn len(&self) -> usize {
        self.seen.len()
    }

    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}

impl Default for ConnTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
//  TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_eth_ipv4_tcp(
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        src_port: u16,
        dst_port: u16,
    ) -> Vec<u8> {
        let mut pkt = vec![0u8; 14 + 20 + 20];
        // Ethernet
        pkt[12] = 0x08; pkt[13] = 0x00;
        // IPv4
        pkt[14] = 0x45;
        let total_len = (20u16 + 20).to_be_bytes();
        pkt[16] = total_len[0]; pkt[17] = total_len[1];
        pkt[22] = 64;
        pkt[23] = PROTO_TCP;
        pkt[26..30].copy_from_slice(&src_ip);
        pkt[30..34].copy_from_slice(&dst_ip);
        let cksum = compute_ipv4_checksum(&pkt[14..34]);
        pkt[24] = cksum[0]; pkt[25] = cksum[1];
        // TCP
        let sp = src_port.to_be_bytes();
        let dp = dst_port.to_be_bytes();
        pkt[34] = sp[0]; pkt[35] = sp[1];
        pkt[36] = dp[0]; pkt[37] = dp[1];
        pkt[46] = 0x50; // data offset = 5
        pkt[47] = 0x02; // SYN
        pkt
    }

    fn compute_ipv4_checksum(header: &[u8]) -> [u8; 2] {
        let mut sum = 0u32;
        for i in (0..header.len()).step_by(2) {
            let word = u16::from_be_bytes([header[i], header[i + 1]]) as u32;
            sum += word;
        }
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        (!sum as u16).to_be_bytes()
    }

    mod ethernet_parsing {
        use super::*;

        #[test]
        fn ethernet_frame_ethertype_is_parsed_correctly() {
            let pkt = make_eth_ipv4_tcp([1, 2, 3, 4], [5, 6, 7, 8], 1234, 80);
            let frame = parse_ethernet(&pkt).unwrap();
            assert_eq!(frame.ethertype, ETHERTYPE_IPV4);
        }

        #[test]
        fn ethernet_too_short_returns_err() {
            assert_eq!(parse_ethernet(&[0u8; 10]), Err(ParseError::TooShort));
        }

        #[test]
        fn unknown_ethertype_returns_err() {
            let mut pkt = vec![0u8; 14];
            pkt[12] = 0xDE;
            pkt[13] = 0xAD;
            assert_eq!(
                parse_ethernet(&pkt),
                Err(ParseError::UnknownEthertype(0xDEAD))
            );
        }
    }

    mod ipv4_parsing {
        use super::*;

        #[test]
        fn ipv4_checksum_validates_correctly() {
            let pkt = make_eth_ipv4_tcp([10, 0, 0, 1], [10, 0, 0, 2], 5000, 80);
            let frame = parse_ethernet(&pkt).unwrap();
            assert!(
                ipv4_checksum(&frame.payload[..20]),
                "checksum should be valid for test fixture"
            );
        }

        #[test]
        fn ipv4_src_and_dst_are_parsed_correctly() {
            let pkt = make_eth_ipv4_tcp([192, 168, 1, 1], [10, 0, 0, 1], 9999, 443);
            let frame = parse_ethernet(&pkt).unwrap();
            let ip = parse_ipv4(frame.payload).unwrap();
            assert_eq!(ip.src, [192, 168, 1, 1]);
            assert_eq!(ip.dst, [10, 0, 0, 1]);
        }
    }

    mod tcp_parsing {
        use super::*;

        #[test]
        fn tcp_flags_are_extracted_correctly() {
            let pkt = make_eth_ipv4_tcp([1, 2, 3, 4], [5, 6, 7, 8], 1234, 80);
            let frame = parse_ethernet(&pkt).unwrap();
            let ip = parse_ipv4(frame.payload).unwrap();
            let tcp = parse_tcp(ip.payload).unwrap();
            assert_eq!(tcp.flags & 0x02, 0x02, "SYN flag should be set");
        }

        #[test]
        fn tcp_ports_are_parsed_correctly() {
            let pkt = make_eth_ipv4_tcp([0, 0, 0, 0], [0, 0, 0, 0], 12345, 443);
            let frame = parse_ethernet(&pkt).unwrap();
            let ip = parse_ipv4(frame.payload).unwrap();
            let tcp = parse_tcp(ip.payload).unwrap();
            assert_eq!(tcp.src_port, 12345);
            assert_eq!(tcp.dst_port, 443);
        }
    }

    mod connection_tracking {
        use super::*;

        #[test]
        fn connection_tracking_identifies_seen_four_tuple() {
            let mut tracker = ConnTracker::new();
            let pkt = make_eth_ipv4_tcp([1, 2, 3, 4], [5, 6, 7, 8], 1111, 80);
            let frame = parse_ethernet(&pkt).unwrap();
            let ip = parse_ipv4(frame.payload).unwrap();
            let tcp = parse_tcp(ip.payload).unwrap();

            assert!(!tracker.observe(&ip, &tcp), "first observation should not be a duplicate");
            assert!(tracker.observe(&ip, &tcp), "second observation should be detected");
        }

        #[test]
        fn different_connections_are_tracked_separately() {
            let mut tracker = ConnTracker::new();
            let p1 = make_eth_ipv4_tcp([1, 2, 3, 4], [5, 6, 7, 8], 1000, 80);
            let p2 = make_eth_ipv4_tcp([1, 2, 3, 4], [5, 6, 7, 8], 2000, 80);

            let f1 = parse_ethernet(&p1).unwrap();
            let i1 = parse_ipv4(f1.payload).unwrap();
            let t1 = parse_tcp(i1.payload).unwrap();

            let f2 = parse_ethernet(&p2).unwrap();
            let i2 = parse_ipv4(f2.payload).unwrap();
            let t2 = parse_tcp(i2.payload).unwrap();

            assert!(!tracker.observe(&i1, &t1));
            assert!(!tracker.observe(&i2, &t2));
            assert_eq!(tracker.len(), 2);
        }
    }
}
