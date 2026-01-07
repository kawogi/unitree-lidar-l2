use std::fmt::{self, Display};

use anyhow::{Context, Result, bail};
use bytes::Buf;
use crc_fast::CrcAlgorithm;

use crate::{
    ToUsize,
    ack::{Ack, LidarAckData},
    command::{Command, LidarCommand},
    imu::LidarImuData,
    point_data::LidarPointData,
    user_ctrl_cmd::{LidarUserCtrlCmd, UserCmd},
    version::{LidarVersionData, Version},
    work_mode::{LidarWorkModeConfig, WorkMode},
};

/// Frame Header
#[repr(C)]
pub(crate) struct FrameHeader {
    /// Head: 0x55 0xAA 0x05 0x0A
    header: [u8; 4],
    /// packet type
    pub(crate) packet_type: u32,
    /// packet size - total bytes of the whole packet
    pub(crate) packet_size: u32,
}

impl FrameHeader {
    pub(crate) const LEN: usize = size_of::<Self>();

    /// Every frame starts with these magic bytes
    const FRAME_HEADER_ARRAY: [u8; 4] = [0x55, 0xAA, 0x05, 0x0A];

    pub(crate) fn parse(bytes: &[u8]) -> Result<(Self, &[u8])> {
        let Some((bytes, remainder)) = bytes.split_at_checked(Self::LEN) else {
            bail!(
                "expected a minimum of {} bytes but got {}",
                Self::LEN,
                bytes.len()
            );
        };

        let Some(mut bytes) = bytes.strip_prefix(&Self::FRAME_HEADER_ARRAY) else {
            bail!("wrong magic bytes");
        };

        let packet_type = bytes.get_u32_le();
        let packet_size = bytes.get_u32_le();

        if !bytes.is_empty() {
            unreachable!("bytes should've been completely consumed");
        }

        Ok((
            Self {
                header: Self::FRAME_HEADER_ARRAY,
                packet_type,
                packet_size,
            },
            remainder,
        ))
    }
}

/**
 * @brief Frame Tail
 * @note 12 bytes
 */
#[repr(C)]
pub(crate) struct FrameTail {
    /// crc check of head and data
    pub(crate) crc32: u32,
    /// msg ack for lidar
    ///
    /// NOTE: all zero for packets coming from the LIDAR; unknown content for packets sent by the host
    msg_type_check: u32,
    /// reserve
    reserve: [u8; 2],
    /// Tail: 0x00 0xFF
    tail: [u8; 2],
}

impl FrameTail {
    pub(crate) const LEN: usize = size_of::<Self>();

    /// Every frame ends with these magic bytes
    const FRAME_TAIL_ARRAY: [u8; 2] = [0x00, 0xFF];

    pub(crate) fn parse(bytes: &[u8]) -> Result<(Self, &[u8])> {
        let Some((mut bytes, remainder)) = bytes.split_at_checked(Self::LEN) else {
            bail!(
                "expected a minimum of {} bytes but got {}",
                Self::LEN,
                bytes.len()
            );
        };

        let crc32 = bytes.get_u32_le();
        let msg_type_check = bytes.get_u32_le();
        let reserve = [bytes.get_u8(), bytes.get_u8()];
        if bytes != Self::FRAME_TAIL_ARRAY {
            bail!("wrong tail");
        }

        Ok((
            Self {
                crc32,
                msg_type_check,
                reserve,
                tail: Self::FRAME_TAIL_ARRAY,
            },
            remainder,
        ))
    }
}

#[repr(u32)]
pub(crate) enum PacketType {
    LidarUserCmd = Self::LIDAR_USER_CMD,
    LidarAckData = Self::LIDAR_ACK_DATA,
    LidarPointData = Self::LIDAR_POINT_DATA,
    Lidar2DPointData = Self::LIDAR_2D_POINT_DATA,
    LidarImuData = Self::LIDAR_IMU_DATA,
    LidarVersion = Self::LIDAR_VERSION,
    LidarTimeStamp = Self::LIDAR_TIME_STAMP,
    LidarWorkModeConfig = Self::LIDAR_WORK_MODE_CONFIG,
    LidarIpAddressConfig = Self::LIDAR_IP_ADDRESS_CONFIG,
    LidarMacAddressConfig = Self::LIDAR_MAC_ADDRESS_CONFIG,
    LidarCommand = Self::LIDAR_COMMAND,
    LidarParamData = Self::LIDAR_PARAM_DATA,
    LidarWorkMode = Self::LIDAR_WORK_MODE,
}

impl PacketType {
    pub(crate) const LIDAR_USER_CMD: u32 = 100;
    pub(crate) const LIDAR_ACK_DATA: u32 = 101;
    pub(crate) const LIDAR_POINT_DATA: u32 = 102;
    pub(crate) const LIDAR_2D_POINT_DATA: u32 = 103;
    pub(crate) const LIDAR_IMU_DATA: u32 = 104;
    pub(crate) const LIDAR_VERSION: u32 = 105;
    pub(crate) const LIDAR_TIME_STAMP: u32 = 106;
    pub(crate) const LIDAR_WORK_MODE_CONFIG: u32 = 107;
    pub(crate) const LIDAR_IP_ADDRESS_CONFIG: u32 = 108;
    pub(crate) const LIDAR_MAC_ADDRESS_CONFIG: u32 = 109;

    pub(crate) const LIDAR_COMMAND: u32 = 2000;
    pub(crate) const LIDAR_PARAM_DATA: u32 = 2001;
    /// TODO this is a guess
    pub(crate) const LIDAR_WORK_MODE: u32 = 2002;
}

impl TryFrom<u32> for PacketType {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            Self::LIDAR_USER_CMD => Ok(Self::LidarUserCmd),
            Self::LIDAR_ACK_DATA => Ok(Self::LidarAckData),
            Self::LIDAR_POINT_DATA => Ok(Self::LidarPointData),
            Self::LIDAR_2D_POINT_DATA => Ok(Self::Lidar2DPointData),
            Self::LIDAR_IMU_DATA => Ok(Self::LidarImuData),
            Self::LIDAR_VERSION => Ok(Self::LidarVersion),
            Self::LIDAR_TIME_STAMP => Ok(Self::LidarTimeStamp),
            Self::LIDAR_WORK_MODE_CONFIG => Ok(Self::LidarWorkModeConfig),
            Self::LIDAR_IP_ADDRESS_CONFIG => Ok(Self::LidarIpAddressConfig),
            Self::LIDAR_MAC_ADDRESS_CONFIG => Ok(Self::LidarMacAddressConfig),
            Self::LIDAR_COMMAND => Ok(Self::LidarCommand),
            Self::LIDAR_PARAM_DATA => Ok(Self::LidarParamData),
            Self::LIDAR_WORK_MODE => Ok(Self::LidarWorkMode),
            unknown => bail!("unknown packet type: {unknown}"),
        }
    }
}

pub enum Packet {
    LidarUserCmd(UserCmd),
    LidarAckData(Ack),
    LidarPointData(Box<LidarPointData>),
    Lidar2DPointData(Vec<u8>),
    LidarImuData(LidarImuData),
    LidarVersion(Version),
    LidarTimeStamp(Vec<u8>),
    LidarWorkModeConfig(WorkMode),
    LidarIpAddressConfig(Vec<u8>),
    LidarMacAddressConfig(Vec<u8>),
    LidarCommand(Command),
    LidarParamData(Vec<u8>),
    LidarWorkMode(WorkMode),
}

impl Packet {
    /// Deserializes a packet from the given input.
    ///
    /// Returns the parsed packet along with the remaining non-consumed bytes.
    ///
    /// # Errors
    ///
    /// Errors if
    ///
    /// - the provided buffer doesn't start with a valid packet
    /// - doesn't contain enough bytes is otherwise
    /// - has a CRC mismatch
    /// - contains illegal values
    pub fn parse(input: &[u8]) -> Result<(Self, &[u8])> {
        let (header, mut remainder) = FrameHeader::parse(input)?;

        let Some(payload_len) = header
            .packet_size
            .to_usize()
            .checked_sub(FrameHeader::LEN + FrameTail::LEN)
        else {
            bail!("packet is too small to hold any payload");
        };

        let payload_bytes = remainder
            .split_off(..payload_len)
            .context("payload truncated")?;

        // println!("payload {}", payload_bytes.len());
        let payload_crc = crc_fast::checksum(CrcAlgorithm::Crc32IsoHdlc, payload_bytes);

        let (tail, remainder) = FrameTail::parse(remainder)?;

        if payload_crc != u64::from(tail.crc32) {
            bail!("CRC mismatch");
        }

        let packet_type = PacketType::try_from(header.packet_type)?;
        let packet = match packet_type {
            PacketType::LidarUserCmd => {
                let (cmd, _) = LidarUserCtrlCmd::parse(payload_bytes)?;
                Self::LidarUserCmd(cmd.try_into()?)
            }
            PacketType::LidarAckData => {
                let (data, _) = LidarAckData::parse(payload_bytes)?;
                Self::LidarAckData(data.try_into()?)
            }
            PacketType::LidarPointData => {
                let (data, _) = LidarPointData::parse(payload_bytes)?;
                Self::LidarPointData(Box::new(data))
            }
            PacketType::Lidar2DPointData => {
                // TODO never seen in the wild so far
                Self::Lidar2DPointData(payload_bytes.to_vec())
            }
            PacketType::LidarImuData => {
                let (data, _) = LidarImuData::parse(payload_bytes)?;
                Self::LidarImuData(data)
            }
            PacketType::LidarVersion => {
                let (data, _) = LidarVersionData::parse(payload_bytes)?;
                Self::LidarVersion(data.try_into()?)
            }
            PacketType::LidarTimeStamp => {
                // TODO never seen in the wild so far
                Self::LidarTimeStamp(payload_bytes.to_vec())
            }
            PacketType::LidarWorkModeConfig => {
                // TODO never seen in the wild so far
                let (config, _) = LidarWorkModeConfig::parse(payload_bytes)?;
                Self::LidarWorkModeConfig(config.try_into()?)
            }
            PacketType::LidarIpAddressConfig => {
                // TODO never seen in the wild so far
                Self::LidarIpAddressConfig(payload_bytes.to_vec())
            }
            PacketType::LidarMacAddressConfig => {
                // TODO never seen in the wild so far
                Self::LidarMacAddressConfig(payload_bytes.to_vec())
            }
            PacketType::LidarCommand => {
                let (command, _) = LidarCommand::parse(payload_bytes)?;
                Self::LidarCommand(command.try_into()?)
            }
            PacketType::LidarParamData => Self::LidarParamData(payload_bytes.to_vec()),
            PacketType::LidarWorkMode => {
                let (config, _) = LidarWorkModeConfig::parse(payload_bytes)?;
                Self::LidarWorkMode(config.try_into()?)
            }
        };

        Ok((packet, remainder))
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Packet::LidarUserCmd(cmd) => write!(f, "UserCmd({cmd})"),
            Packet::LidarAckData(ack) => write!(f, "AckData({ack})"),
            Packet::LidarPointData(data) => write!(f, "PointData({data})"),
            Packet::Lidar2DPointData(raw) => write!(f, "2DPointData({})", raw.len()),
            Packet::LidarImuData(data) => write!(f, "ImuData({data})"),
            Packet::LidarVersion(version) => write!(f, "Version({version})"),
            Packet::LidarTimeStamp(raw) => write!(f, "TimeStamp({})", raw.len()),
            Packet::LidarWorkModeConfig(config) => write!(f, "WorkModeConfig({config})"),
            Packet::LidarIpAddressConfig(raw) => write!(f, "IpAddressConfig({})", raw.len()),
            Packet::LidarMacAddressConfig(raw) => write!(f, "MacAddressConfig({})", raw.len()),
            Packet::LidarCommand(command) => write!(f, "Command({command})"),
            Packet::LidarParamData(raw) => write!(f, "ParamData({})", raw.len()),
            Packet::LidarWorkMode(mode) => write!(f, "WorkMode({mode})"),
        }
    }
}
