#![allow(unused_crate_dependencies, reason = "used in examples")]

mod ack;
mod command;
mod frame;
mod imu;
mod info;
mod user_ctrl_cmd;
mod version;
mod work_mode;

pub use frame::Packet;

use crate::info::DataInfo;

// compile-time check to ensure we're not running on a 16-bit system
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
 * @brief Lidar calib param
 * @note 32 bytes
 */
#[repr(C)]
pub(crate) struct LidarCalibParam {
    /// unit: m
    a_axis_dist: f32,
    /// unit: m
    b_axis_dist: f32,
    /// unit: rad
    theta_angle_bias: f32,
    /// unit: rad
    alpha_angle_bias: f32,
    /// unit: rad
    beta_angle: f32,
    /// unit: rad
    xi_angle: f32,
    /// unit: mm
    range_bias: f32,
    /// unit: 1
    range_scale: f32,
}

/**
 * @brief Lidar Inside State
 * @note 36 bytes
 */
#[repr(C)]
pub(crate) struct LidarInsideState {
    /// Up motor rotation period
    sys_rotation_period: u32,
    /// Down motor rotation period
    com_rotation_period: u32,
    dirty_index: f32,
    packet_lost_up: f32,
    packet_lost_down: f32,
    apd_temperature: f32,
    apd_voltage: f32,
    laser_voltage: f32,
    imu_temperature: f32,
}

/**
 * @brief Lidar Point Data
 * @note 1020 bytes
 */
#[repr(C)]
pub(crate) struct LidarPointData {
    /// Packet Info
    info: DataInfo,

    /// Lidar inside state
    state: LidarInsideState,

    /// Lidar calib param
    param: LidarCalibParam,

    // Line info
    /// Horizontal Start Angle
    com_horizontal_angle_start: f32,
    /// Horizontal Angle Step
    com_horizontal_angle_step: f32,
    /// Scan period [second]
    scan_period: f32,
    /// Minimum range value [m]
    range_min: f32,
    /// Maximum range value [m]
    range_max: f32,
    /// First Angle [rad]
    angle_min: f32,
    /// Angle Step [rad]
    angle_increment: f32,
    /// Time step [second]
    time_increment: f32,
    /// Point Number
    point_num: u32,
    /// Point Distance [mm]
    ranges: [u16; 300],
    /// Point Reflect [0-255]
    intensities: [u8; 300],
}

/**
 * @brief Lidar 2D Point Data
 * @note 5512 bytes
 */
#[repr(C)]
pub(crate) struct Lidar2DPointData {
    /// Packet Info
    info: DataInfo,

    /// Lidar inside state
    state: LidarInsideState,

    /// Lidar calib param
    param: LidarCalibParam,

    // Line info
    /// scan period [second]
    scan_period: f32,
    /// minimum range value [m]
    range_min: f32,
    /// maximum range value [m]
    range_max: f32,
    /// First Angle [rad]
    angle_min: f32,
    /// Angle Step [rad]
    angle_increment: f32,
    /// point time step
    time_increment: f32,
    /// Point Number
    point_num: u32,
    /// Point Distance Data
    ranges: [u16; 1800],
    /// Point Reflect Data
    intensities: [u8; 1800],
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
