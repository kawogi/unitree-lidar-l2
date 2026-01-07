#![allow(unused_crate_dependencies, reason = "used in examples")]

mod ack;
mod command;
mod frame;
mod imu;
mod info;
mod point_data;
mod user_ctrl_cmd;
mod version;
mod work_mode;

pub use frame::Packet;

/// compile-time check to ensure we're not running on a 16-bit system
const _CHECK32: () = assert!(usize::BITS >= u32::BITS, "16 bit platforms are unsupported");

// provides a hassle-free conversion to usize
trait ToUsize {
    fn to_usize(self) -> usize;
}

impl ToUsize for u32 {
    fn to_usize(self) -> usize {
        usize::try_from(self).unwrap_or_else(|error| unreachable!("failed to convert to usize despite being on a system with sufficient word width: {error}"))
    }
}

/**
 * @brief Lidar IP Config
 * @note 20 bytes
 */
#[repr(C)]
pub(crate) struct LidarIpAddressConfig {
    /// UDP local ip
    lidar_ip: [u8; 4],
    /// UDP remote ip
    user_ip: [u8; 4],
    /// Gate way
    gateway: [u8; 4],
    /// Subnet mask
    subnet_mask: [u8; 4],
    /// UDP local port
    lidar_port: u16,
    /// UDP remote port
    user_port: u16,
}

/**
 * @brief Lidar MAC address Config
 * @note 8 bytes
 */
#[repr(C)]
pub(crate) struct LidarMacAddressConfig {
    mac: [u8; 6],
    reserve: [u8; 2],
}
