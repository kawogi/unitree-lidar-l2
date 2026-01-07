use std::fmt::{self, Display};

use anyhow::{Result, bail};
use bytes::Buf;

pub enum UserCmd {
    ResetType(u32),
    StandbyType(StandbyType),
    VersionGet(u32),
    LatencyType(u32),
    ConfigReset(u32),
    ConfigGet(u32),
    ConfigAutoStandby(u32),
}

impl UserCmd {
    const RESET_TYPE: u32 = 1;
    const STANDBY_TYPE: u32 = 2;
    const VERSION_GET: u32 = 3;
    const LATENCY_TYPE: u32 = 4;
    const CONFIG_RESET: u32 = 5;
    const CONFIG_GET: u32 = 6;
    const CONFIG_AUTO_STANDBY: u32 = 7;
}

impl TryFrom<LidarUserCtrlCmd> for UserCmd {
    type Error = anyhow::Error;

    fn try_from(cmd: LidarUserCtrlCmd) -> Result<Self, Self::Error> {
        (cmd.cmd_type, cmd.cmd_value).try_into()
    }
}

impl TryFrom<(u32, u32)> for UserCmd {
    type Error = anyhow::Error;

    fn try_from((typ, value): (u32, u32)) -> Result<Self, Self::Error> {
        match typ {
            Self::RESET_TYPE => Ok(Self::ResetType(value)),
            Self::STANDBY_TYPE => Ok(Self::StandbyType(value.try_into()?)),
            Self::VERSION_GET => Ok(Self::VersionGet(value)),
            Self::LATENCY_TYPE => Ok(Self::LatencyType(value)),
            Self::CONFIG_RESET => Ok(Self::ConfigReset(value)),
            Self::CONFIG_GET => Ok(Self::ConfigGet(value)),
            Self::CONFIG_AUTO_STANDBY => Ok(Self::ConfigAutoStandby(value)),
            unknown => bail!("unknown command type: {unknown}"),
        }
    }
}

impl Display for UserCmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserCmd::ResetType(value) => write!(f, "ResetType({value})"),
            UserCmd::StandbyType(value) => write!(f, "StandbyType({value})"),
            UserCmd::VersionGet(value) => write!(f, "VersionGet({value})"),
            UserCmd::LatencyType(value) => write!(f, "LatencyType({value})"),
            UserCmd::ConfigReset(value) => write!(f, "ConfigReset({value})"),
            UserCmd::ConfigGet(value) => write!(f, "ConfigGet({value})"),
            UserCmd::ConfigAutoStandby(value) => write!(f, "ConfigAutoStandby({value})"),
        }
    }
}

pub enum StandbyType {
    Start = 0,
    Standby = 1,
}

impl TryFrom<u32> for StandbyType {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Start),
            1 => Ok(Self::Standby),
            unknown => bail!("unknown standby mode: {unknown}"),
        }
    }
}

impl Display for StandbyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            StandbyType::Start => "Start",
            StandbyType::Standby => "Standby",
        };

        f.write_str(str)
    }
}
/**
 * @brief Lidar User Control Command
 * @note 8 bytes
 */
#[repr(C)]
pub struct LidarUserCtrlCmd {
    ///   0:null, 1:standby
    cmd_type: u32,
    cmd_value: u32,
}

impl LidarUserCtrlCmd {
    pub(crate) const LEN: usize = size_of::<Self>();

    pub(crate) fn parse(bytes: &[u8]) -> Result<(Self, &[u8])> {
        let Some((mut bytes, remainder)) = bytes.split_at_checked(Self::LEN) else {
            bail!(
                "expected a minimum of {} bytes but got {}",
                Self::LEN,
                bytes.len()
            );
        };

        let cmd_type = bytes.get_u32_le();
        let cmd_value = bytes.get_u32_le();

        if !bytes.is_empty() {
            unreachable!("bytes should've been completely consumed");
        }

        Ok((
            Self {
                cmd_type,
                cmd_value,
            },
            remainder,
        ))
    }
}

impl Display for LidarUserCtrlCmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            cmd_type,
            cmd_value,
        } = self;
        write!(f, "type:{cmd_type}, value:{cmd_value}")
    }
}
