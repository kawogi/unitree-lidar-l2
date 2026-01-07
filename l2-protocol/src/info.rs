use std::fmt::{self, Display};

use anyhow::{Result, bail};
use bytes::Buf;


/**
 * @brief Time stamp
 * @note 8 bytes
 */
#[repr(C)]
pub(crate) struct TimeStamp {
    /// time stamp of second
    sec: u32,
    /// time stamp of nsecond
    nsec: u32,
}

impl TimeStamp {
    pub(crate) const LEN: usize = size_of::<Self>();

    pub(crate) fn parse(bytes: &[u8]) -> Result<(Self, &[u8])> {
        let Some((mut bytes, remainder)) = bytes.split_at_checked(Self::LEN) else {
            bail!(
                "expected a minimum of {} bytes but got {}",
                Self::LEN,
                bytes.len()
            );
        };

        let sec = bytes.get_u32_le();
        let nsec = bytes.get_u32_le();

        Ok((Self { sec, nsec }, remainder))
    }
}
impl Display for TimeStamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:09}", self.sec, self.nsec)
    }
}

/**
 * @brief Data Info
 * @note 16 bytes
 */
#[repr(C)]
pub(crate) struct DataInfo {
    /// packet sequence id, consecutively increasing
    seq: u32,
    /// Packet Size
    payload_size: u32,
    /// timestamp
    stamp: TimeStamp,
}

impl DataInfo {
    pub(crate) const LEN: usize = size_of::<Self>();

    pub(crate) fn parse(bytes: &[u8]) -> Result<(Self, &[u8])> {
        let Some((mut bytes, remainder)) = bytes.split_at_checked(Self::LEN) else {
            bail!(
                "expected a minimum of {} bytes but got {}",
                Self::LEN,
                bytes.len()
            );
        };

        let seq = bytes.get_u32_le();
        let payload_size = bytes.get_u32_le();
        let (stamp, bytes) = TimeStamp::parse(bytes)?;

        if !bytes.is_empty() {
            unreachable!("all bytes should've been consumed");
        }

        Ok((
            Self {
                seq,
                payload_size,
                stamp,
            },
            remainder,
        ))
    }
}

impl Display for DataInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{} len:{}, time:{}",
            self.seq, self.payload_size, self.stamp
        )
    }
}
