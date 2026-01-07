use std::fmt::{self, Display};

use anyhow::{Result, bail};
use bytes::Buf;

#[expect(
    clippy::struct_excessive_bools,
    reason = "this represents a configuration bit-field"
)]
pub struct WorkMode {
    wide_angle: bool,
    measure_2d: bool,
    disable_imu: bool,
    serial_mode: bool,
    wait_start: bool,
}

impl TryFrom<LidarWorkModeConfig> for WorkMode {
    type Error = anyhow::Error;

    fn try_from(value: LidarWorkModeConfig) -> Result<Self, Self::Error> {
        let LidarWorkModeConfig { mode: flags } = value;
        // Bit Position	Function	Value 0	Value 1
        // 3
        // 4
        // 5-31	Reserved	Reserved	Reserved

        if flags & 0b1111_1111_1111_1111_1111_1111_1110_0000 != 0 {
            bail!("unknown mode flags: {value}")
        }

        // Bit 0: Switch between standard FOV and wide-angle FOV
        // 0: Standard FOV (180°)
        // 1: Wide-angle FOV (192°)
        let wide_angle = flags & 0b0000_0001 != 0;

        // Bit 1: Switch between 3D and 2D measurement modes
        // 0: 3D measurement mode
        // 1: 2D measurement mode
        let measure_2d = flags & 0b0000_0010 != 0;

        // Bit 2: Enable or disable IMU
        // 0: Enable IMU
        // 1: Disable IMU
        let disable_imu = flags & 0b0000_0100 != 0;

        // Bit 3: Switch between Ethernet mode and serial mode
        // 0: Ethernet mode
        // 1: Serial mode
        let serial_mode = flags & 0b0000_1000 != 0;

        // Bit 4: Switch between lidar power-on default start mode
        // 0: Power on and start automatically
        // 1: Power on and wait for start command without rotation
        let wait_start = flags & 0b0001_0000 != 0;

        // TODO
        Ok(Self {
            wide_angle,
            measure_2d,
            disable_imu,
            serial_mode,
            wait_start,
        })
    }
}

impl Display for WorkMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WorkMode(angle:{}, {}, imu:{}, interface:{}, start:{})",
            if self.wide_angle { "192°" } else { "180°" },
            if self.measure_2d { "2D" } else { "3D" },
            if self.disable_imu { "off" } else { "on" },
            if self.serial_mode {
                "serial"
            } else {
                "ethernet"
            },
            if self.wait_start { "manual" } else { "auto" },
        )?;

        Ok(())
    }
}

#[repr(C)]
pub(crate) struct LidarWorkModeConfig {
    mode: u32,
}

impl LidarWorkModeConfig {
    pub(crate) const LEN: usize = size_of::<Self>();

    pub(crate) fn parse(bytes: &[u8]) -> Result<(Self, &[u8])> {
        let Some((mut bytes, remainder)) = bytes.split_at_checked(Self::LEN) else {
            bail!(
                "expected a minimum of {} bytes but got {}",
                Self::LEN,
                bytes.len()
            );
        };

        let flags = bytes.get_u32_le();

        if !bytes.is_empty() {
            unreachable!("bytes should've been completely consumed");
        }

        Ok((Self { mode: flags }, remainder))
    }
}

impl Display for LidarWorkModeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { mode: flags } = self;
        write!(f, "flags:{flags:#034b}")
    }
}
