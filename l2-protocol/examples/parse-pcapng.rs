#![allow(unused_crate_dependencies, reason = "used in library")]
#![expect(
    clippy::unwrap_used,
    clippy::todo,
    clippy::panic,
    reason = "ok for throw-away demo code"
)]

use etherparse::{EtherType, IpNumber};
use l2_protocol::Packet;
use pcap_parser::traits::PcapReaderIterator;
use pcap_parser::{Block, PcapBlockOwned, PcapError, PcapNGReader};
use std::fs::File;
use std::net::Ipv4Addr;

fn main() {
    let file = File::open("example_lidar_udp.pcapng").unwrap();
    let mut packet_index = 0;
    let mut reader = PcapNGReader::new(0x0001_0000, file).expect("PcapNGReader");
    loop {
        // if num_blocks > 10 {
        //     break;
        // }
        match reader.next() {
            Ok((offset, block)) => {
                // println!("got new block");
                match block {
                    PcapBlockOwned::Legacy(_) => todo!(),
                    PcapBlockOwned::LegacyHeader(_) => todo!(),
                    PcapBlockOwned::NG(block) => match block {
                        Block::SectionHeader(section_header_block) => {
                            println!("section header: {section_header_block:?}");
                        }
                        Block::InterfaceDescription(interface_description_block) => {
                            println!("interface description: {interface_description_block:?}");
                        }
                        Block::EnhancedPacket(enhanced_packet_block) => {
                            packet_index += 1;

                            let ethernet = etherparse::Ethernet2Slice::from_slice_without_fcs(
                                enhanced_packet_block.data,
                            )
                            .unwrap();
                            if ethernet.ether_type() == EtherType::IPV4 {
                                let ipv4 =
                                    etherparse::Ipv4Slice::from_slice(ethernet.payload().payload)
                                        .unwrap();
                                if ipv4.header().protocol() == IpNumber::UDP {
                                    let udp =
                                        etherparse::UdpSlice::from_slice(ipv4.payload().payload)
                                            .unwrap();

                                    if ipv4.header().source_addr()
                                        == Ipv4Addr::from_octets([192, 168, 1, 62])
                                    {
                                        if udp.source_port() == 6101 {
                                            parse_incoming(udp.payload(), packet_index);
                                        }
                                    } else if udp.destination_port() == 6101 {
                                        parse_outgoing(udp.payload(), packet_index);
                                    }
                                    // }
                                }
                            }
                        }
                        Block::SimplePacket(_) => todo!(),
                        Block::NameResolution(_) => todo!(),
                        Block::InterfaceStatistics(interface_statistics_block) => {
                            println!("interface statistics: {interface_statistics_block:?}");
                        }
                        Block::SystemdJournalExport(_) => todo!(),
                        Block::DecryptionSecrets(_) => todo!(),
                        Block::ProcessInformation(_) => todo!(),
                        Block::Custom(_) => todo!(),
                        Block::Unknown(_) => todo!(),
                    },
                }
                reader.consume(offset);
            }
            Err(PcapError::Eof) => break,
            Err(PcapError::Incomplete(_)) => {
                reader.refill().unwrap();
            }
            Err(error) => panic!("error while reading: {error:?}"),
        }
    }
}

fn parse_incoming(mut data: &[u8], packet_index: u64) {
    while !data.is_empty() {
        let packet;
        let len = data.len();
        (packet, data) = Packet::parse(data).unwrap();
        if !matches!(packet, Packet::LidarImuData(_)) {
            println!(
                "#{packet_index} LIDAR→USER {packet} payload {} bytes",
                len - data.len() - 24
            );
        }
    }
}

fn parse_outgoing(mut data: &[u8], packet_index: u64) {
    while !data.is_empty() {
        let packet;
        let len = data.len();
        (packet, data) = Packet::parse(data).unwrap();
        println!(
            "#{packet_index} USER→LIDAR {packet} payload {} bytes",
            len - data.len() - 24
        );
    }
}
