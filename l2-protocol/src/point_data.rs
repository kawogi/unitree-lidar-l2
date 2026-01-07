use std::{
    array,
    fmt::{self, Display},
};

use anyhow::{Result, bail};
use bytes::Buf;

use crate::info::DataInfo;

/**
 * @brief Lidar calib param
 * @note 32 bytes
 */
#[repr(C)]
#[derive(Debug)]
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

impl LidarCalibParam {
    pub(crate) const LEN: usize = size_of::<Self>();

    pub(crate) fn parse(bytes: &[u8]) -> Result<(Self, &[u8])> {
        let Some((mut bytes, remainder)) = bytes.split_at_checked(Self::LEN) else {
            bail!(
                "expected a minimum of {} bytes but got {}",
                Self::LEN,
                bytes.len()
            );
        };

        let a_axis_dist = bytes.get_f32_le();
        let b_axis_dist = bytes.get_f32_le();
        let theta_angle_bias = bytes.get_f32_le();
        let alpha_angle_bias = bytes.get_f32_le();
        let beta_angle = bytes.get_f32_le();
        let xi_angle = bytes.get_f32_le();
        let range_bias = bytes.get_f32_le();
        let range_scale = bytes.get_f32_le();

        if !bytes.is_empty() {
            unreachable!("bytes should've been completely consumed");
        }

        Ok((
            Self {
                a_axis_dist,
                b_axis_dist,
                theta_angle_bias,
                alpha_angle_bias,
                beta_angle,
                xi_angle,
                range_bias,
                range_scale,
            },
            remainder,
        ))
    }
}

impl Display for LidarCalibParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/**
 * @brief Lidar Inside State
 * @note 36 bytes
 */
#[repr(C)]
#[derive(Debug)]
pub(crate) struct LidarInsideState {
    /// The speed of the horizontal low-speed motor, in revolutions per minute (r/min).
    /// Up motor rotation period
    sys_rotation_period: u32,
    /// The speed of the vertical high-speed motor, in revolutions per minute (r/min).
    /// Down motor rotation period
    com_rotation_period: u32,
    /// The index of dirt on the radar's optical surface.
    dirty_index: f32,
    /// The packet loss rate of the upper board of the radar.
    packet_lost_up: f32,
    /// The packet loss rate of the lower board of the radar.
    packet_lost_down: f32,
    /// The temperature of the APD, in degrees Celsius (℃).
    ///
    /// APD: Avalanche Photo Diode (likely)
    apd_temperature: f32,
    /// The voltage of the APD, in Volts (V).
    ///
    /// APD: Avalanche Photo Diode (likely)
    apd_voltage: f32,
    /// The voltage of the laser emitter, in Volts (V).
    laser_voltage: f32,
    /// The temperature of the IMU, in Volts (V) (sic)
    imu_temperature: f32,
}

impl LidarInsideState {
    pub(crate) const LEN: usize = size_of::<Self>();

    pub(crate) fn parse(bytes: &[u8]) -> Result<(Self, &[u8])> {
        let Some((mut bytes, remainder)) = bytes.split_at_checked(Self::LEN) else {
            bail!(
                "expected a minimum of {} bytes but got {}",
                Self::LEN,
                bytes.len()
            );
        };

        let sys_rotation_period = bytes.get_u32_le();
        let com_rotation_period = bytes.get_u32_le();
        let dirty_index = bytes.get_f32_le();
        let packet_lost_up = bytes.get_f32_le();
        let packet_lost_down = bytes.get_f32_le();
        let apd_temperature = bytes.get_f32_le();
        let apd_voltage = bytes.get_f32_le();
        let laser_voltage = bytes.get_f32_le();
        let imu_temperature = bytes.get_f32_le();

        if !bytes.is_empty() {
            unreachable!("bytes should've been completely consumed");
        }

        Ok((
            Self {
                sys_rotation_period,
                com_rotation_period,
                dirty_index,
                packet_lost_up,
                packet_lost_down,
                apd_temperature,
                apd_voltage,
                laser_voltage,
                imu_temperature,
            },
            remainder,
        ))
    }
}

impl Display for LidarInsideState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/**
 * @brief Lidar Point Data
 * @note 1020 bytes
 */
#[repr(C)]
#[derive(Debug)]

pub struct LidarPointData {
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

impl LidarPointData {
    pub(crate) const LEN: usize = size_of::<Self>();

    pub(crate) fn parse(bytes: &[u8]) -> Result<(Self, &[u8])> {
        let Some((bytes, remainder)) = bytes.split_at_checked(Self::LEN) else {
            bail!(
                "expected a minimum of {} bytes but got {}",
                Self::LEN,
                bytes.len()
            );
        };

        let (info, bytes) = DataInfo::parse(bytes)?;
        let (state, bytes) = LidarInsideState::parse(bytes)?;
        let (param, mut bytes) = LidarCalibParam::parse(bytes)?;

        let com_horizontal_angle_start = bytes.get_f32_le();
        let com_horizontal_angle_step = bytes.get_f32_le();
        let scan_period = bytes.get_f32_le();
        let range_min = bytes.get_f32_le();
        let range_max = bytes.get_f32_le();
        let angle_min = bytes.get_f32_le();
        let angle_increment = bytes.get_f32_le();
        let time_increment = bytes.get_f32_le();
        let point_num = bytes.get_u32_le();
        let ranges = array::from_fn(|_| bytes.get_u16_le());
        let intensities = array::from_fn(|_| bytes.get_u8());

        if !bytes.is_empty() {
            unreachable!("bytes should've been completely consumed");
        }

        Ok((
            Self {
                info,
                state,
                param,
                com_horizontal_angle_start,
                com_horizontal_angle_step,
                scan_period,
                range_min,
                range_max,
                angle_min,
                angle_increment,
                time_increment,
                point_num,
                ranges,
                intensities,
            },
            remainder,
        ))
    }
}

impl Display for LidarPointData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
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
