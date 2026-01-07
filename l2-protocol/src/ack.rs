use std::fmt::{self, Display};

use anyhow::{Result, bail};
use bytes::Buf;

use crate::{command::Command, frame::PacketType, user_ctrl_cmd::UserCmd};

#[repr(u32)]
pub enum AckStatus {
    Success = Self::SUCCESS,
    CrcError = Self::CRC_ERROR,
    HeaderError = Self::HEADER_ERROR,
    BlockError = Self::BLOCK_ERROR,
    WaitError = Self::WAIT_ERROR,
}

impl AckStatus {
    pub(crate) const SUCCESS: u32 = 1;
    pub(crate) const CRC_ERROR: u32 = 2;
    pub(crate) const HEADER_ERROR: u32 = 3;
    pub(crate) const BLOCK_ERROR: u32 = 4;
    /// data is not ready
    pub(crate) const WAIT_ERROR: u32 = 5;
}

impl TryFrom<u32> for AckStatus {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            Self::SUCCESS => Ok(Self::Success),
            Self::CRC_ERROR => Ok(Self::CrcError),
            Self::HEADER_ERROR => Ok(Self::HeaderError),
            Self::BLOCK_ERROR => Ok(Self::BlockError),
            Self::WAIT_ERROR => Ok(Self::WaitError),
            unknown => bail!("unknown ack status: {unknown}"),
        }
    }
}

impl Display for AckStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            AckStatus::Success => "Success",
            AckStatus::CrcError => "CrcError",
            AckStatus::HeaderError => "HeaderError",
            AckStatus::BlockError => "BlockError",
            AckStatus::WaitError => "WaitError",
        };
        f.write_str(str)
    }
}

pub enum Ack {
    UserCmd {
        cmd: UserCmd,
        status: AckStatus,
    },
    Command {
        cmd: Command,
        status: AckStatus,
    },
    WorkMode {
        /// likely always 0
        cmd_type: u32,
        /// likely always 0
        cmd_value: u32,
        status: AckStatus,
    },
}

impl TryFrom<LidarAckData> for Ack {
    type Error = anyhow::Error;

    fn try_from(value: LidarAckData) -> Result<Self, Self::Error> {
        let LidarAckData {
            packet_type,
            cmd_type,
            cmd_value,
            status,
        } = value;

        let status = status.try_into()?;

        match packet_type {
            PacketType::LIDAR_USER_CMD => Ok(Self::UserCmd {
                cmd: (cmd_type, cmd_value).try_into()?,
                status,
            }),
            PacketType::LIDAR_COMMAND => Ok(Self::Command {
                cmd: (cmd_type, cmd_value).try_into()?,
                status,
            }),
            PacketType::LIDAR_WORK_MODE => Ok(Self::WorkMode {
                cmd_type,
                cmd_value,
                status,
            }),
            unknown => bail!("ack for unknown packet type: {unknown}"),
        }
    }
}

impl Display for Ack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ack::UserCmd { cmd, status } => write!(f, "Ack::UserCmd({cmd}, {status})"),
            Ack::Command { cmd, status } => write!(f, "Ack::Command({cmd}, {status})"),
            Ack::WorkMode {
                cmd_type,
                cmd_value,
                status,
            } => write!(
                f,
                "Ack::WorkMode(type:{cmd_type}, value:{cmd_value}, {status})"
            ),
        }
    }
}

/**
 * @brief ACK
 * @note Lidar will respond with an ack packet if it receive a packet from user
 * @note 16 bytes
 */
#[repr(C)]
pub(crate) struct LidarAckData {
    /// packet type received by lidar
    packet_type: u32,
    /// cmd type received by lidar
    cmd_type: u32,
    /// cmd value received by lidar
    cmd_value: u32,
    /// execute result
    status: u32,
}

impl LidarAckData {
    pub(crate) const LEN: usize = size_of::<Self>();

    pub(crate) fn parse(bytes: &[u8]) -> Result<(Self, &[u8])> {
        let Some((mut bytes, remainder)) = bytes.split_at_checked(Self::LEN) else {
            bail!(
                "expected a minimum of {} bytes but got {}",
                Self::LEN,
                bytes.len()
            );
        };

        let packet_type = bytes.get_u32_le();
        let cmd_type = bytes.get_u32_le();
        let cmd_value = bytes.get_u32_le();
        let status = bytes.get_u32_le();

        if !bytes.is_empty() {
            unreachable!("bytes should've been completely consumed");
        }

        Ok((
            Self {
                packet_type,
                cmd_type,
                cmd_value,
                status,
            },
            remainder,
        ))
    }
}

impl Display for LidarAckData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            packet_type,
            cmd_type,
            cmd_value,
            status,
        } = self;
        write!(
            f,
            "packet_type:{packet_type}, cmd_type:{cmd_type}, cmd_value:{cmd_value}, status:{status}"
        )
    }
}
