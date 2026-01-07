use std::{
    fmt::{self, Display},
    io::Read,
};

use anyhow::{Context, Ok, Result, bail};

pub struct Version {
    /// hardware version
    hardware: [u8; 4],
    /// software version
    software: [u8; 4],
    /// device name
    name: String,
    /// device compile date
    date: String,
}

impl TryFrom<LidarVersionData> for Version {
    type Error = anyhow::Error;

    fn try_from(value: LidarVersionData) -> Result<Self, Self::Error> {
        let LidarVersionData {
            hw_version,
            sw_version,
            name,
            date,
            reserve: _,
        } = value;

        // let hardware = format!(
        //     "{}.{}.{}.{}",
        //     hw_version[0], hw_version[1], hw_version[2], hw_version[3]
        // );

        // let software = format!(
        //     "{}.{}.{}.{}",
        //     sw_version[0], sw_version[1], sw_version[2], sw_version[3]
        // );

        let mut name = name.as_slice();
        while let [rest @ .., last] = name {
            if *last == 0 {
                name = rest;
            } else {
                break;
            }
        }
        let name =
            String::from_utf8(name.to_vec()).context("device name contained invalid utf-8")?;

        // TODO add some sanity checks
        let date = format!(
            "20{}{}-{}{}-{}{}",
            char::from(date[0]),
            char::from(date[1]),
            char::from(date[2]),
            char::from(date[3]),
            char::from(date[4]),
            char::from(date[5])
        );

        Ok(Self {
            hardware: hw_version,
            software: sw_version,
            name,
            date,
        })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "hw:{}.{}.{}.{}, sw:{}.{}.{}.{}, name:'{}', compiled:'{}'",
            self.hardware[0],
            self.hardware[1],
            self.hardware[2],
            self.hardware[3],
            self.software[0],
            self.software[1],
            self.software[2],
            self.software[3],
            self.name,
            self.date
        )
    }
}

/**
 * @brief Lidar version
 * @note 80 bytes
 */
#[repr(C)]
#[derive(Debug)]
pub struct LidarVersionData {
    /// hardware version
    hw_version: [u8; 4],
    /// software version
    sw_version: [u8; 4],
    /// device name
    name: [u8; 24],
    /// device compile date
    date: [u8; 8],
    reserve: [u8; 40],
}

impl LidarVersionData {
    pub(crate) const LEN: usize = size_of::<Self>();

    pub(crate) fn parse(bytes: &[u8]) -> Result<(Self, &[u8])> {
        let Some((mut bytes, remainder)) = bytes.split_at_checked(Self::LEN) else {
            bail!(
                "expected a minimum of {} bytes but got {}",
                Self::LEN,
                bytes.len()
            );
        };

        let mut hw_version = [0; 4];
        bytes
            .read_exact(&mut hw_version)
            .context("failed to read hw_version")?;

        let mut sw_version = [0; 4];
        bytes
            .read_exact(&mut sw_version)
            .context("failed to read sw_version")?;

        let mut name = [0; 24];
        bytes.read_exact(&mut name).context("failed to read name")?;

        let mut date = [0; 8];
        bytes.read_exact(&mut date).context("failed to read date")?;

        let mut reserve = [0; 40];
        bytes
            .read_exact(&mut reserve)
            .context("failed to read reserve")?;

        if !bytes.is_empty() {
            unreachable!("bytes should've been completely consumed");
        }

        Ok((
            Self {
                hw_version,
                sw_version,
                name,
                date,
                reserve,
            },
            remainder,
        ))
    }
}

impl Display for LidarVersionData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
