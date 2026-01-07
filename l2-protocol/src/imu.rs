use std::fmt::{self, Display};

use anyhow::{Result, bail};
use bytes::Buf;

use crate::info::DataInfo;

// @note 56 bytes
#[repr(C)]
pub struct LidarImuData {
    info: DataInfo,
    /// Quaternion Array.
    quaternion: [f32; 4],
    /// Three-axis angular velocity values.
    angular_velocity: [f32; 3],
    /// Three-axis acceleration values.
    linear_acceleration: [f32; 3],
}

impl LidarImuData {
    pub(crate) const LEN: usize = size_of::<Self>();

    pub(crate) fn parse(bytes: &[u8]) -> Result<(Self, &[u8])> {
        let Some((bytes, remainder)) = bytes.split_at_checked(Self::LEN) else {
            bail!(
                "expected a minimum of {} bytes but got {}",
                Self::LEN,
                bytes.len()
            );
        };

        let (info, mut bytes) = DataInfo::parse(bytes)?;

        let quaternion = [
            bytes.get_f32_le(),
            bytes.get_f32_le(),
            bytes.get_f32_le(),
            bytes.get_f32_le(),
        ];
        let angular_velocity = [bytes.get_f32_le(), bytes.get_f32_le(), bytes.get_f32_le()];
        let linear_acceleration = [bytes.get_f32_le(), bytes.get_f32_le(), bytes.get_f32_le()];

        if !bytes.is_empty() {
            unreachable!("bytes should've been completely consumed");
        }

        Ok((
            Self {
                info,
                quaternion,
                angular_velocity,
                linear_acceleration,
            },
            remainder,
        ))
    }
}

impl Display for LidarImuData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "info:{}, quat:[{}, {}, {}, {}], ang_vel:[{}, {}, {}], accel:[{}, {}, {}]",
            self.info,
            self.quaternion[0],
            self.quaternion[1],
            self.quaternion[2],
            self.quaternion[3],
            self.angular_velocity[0],
            self.angular_velocity[1],
            self.angular_velocity[2],
            self.linear_acceleration[0],
            self.linear_acceleration[1],
            self.linear_acceleration[2],
        )
    }
}
