//! Utilities for parsing and formatting RFC 3339 timestamps.
//!
//! Serenity uses `jiff::Timestamp` to represent timestamps. Conversions are provided between id
//! types and `Timestamp`. For convenience, `jiff::Timestamp` is exported from this module.

use std::fmt;

pub use jiff::Timestamp;

/// Discord's epoch starts at "2015-01-01T00:00:00+00:00"
const DISCORD_EPOCH: u64 = 1_420_070_400_000;
// `(u64::MAX >> 22) + DISCORD_EPOCH` = 5818116911103 = "Wed May 15 2154 07:35:11 GMT+0000"
const MAX_DISCORD_ID_MILLIS: u64 = 5_818_116_911_103;

/// This trait is used by methods on the id types.
pub(crate) trait DiscordIds {
    fn from_discord_id(id: u64) -> Self;
    fn try_as_discord_id(self) -> Result<u64, TimestampOutOfRange>;
}

impl DiscordIds for Timestamp {
    fn from_discord_id(id: u64) -> Self {
        // This can't fail because of the bit shifting
        // `(u64::MAX >> 22) + DISCORD_EPOCH` = 5818116911103 = "Wed May 15 2154 07:35:11 GMT+0000"
        Self::from_millisecond(((id >> 22) + DISCORD_EPOCH) as i64).expect("can't fail")
    }

    fn try_as_discord_id(self) -> Result<u64, TimestampOutOfRange> {
        let unix_millis =
            TryInto::<u64>::try_into(self.as_millisecond()).map_err(|_| TimestampOutOfRange)?;
        if !(DISCORD_EPOCH..=MAX_DISCORD_ID_MILLIS).contains(&unix_millis) {
            return Err(TimestampOutOfRange);
        }
        Ok((unix_millis - DISCORD_EPOCH) << 22)
    }
}

/// Error which could be raised when converting a Timestamp to a Discord id.
#[derive(Debug)]
pub struct TimestampOutOfRange;

impl std::error::Error for TimestampOutOfRange {}

impl fmt::Display for TimestampOutOfRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("timestamp is outside the valid range of Discord id timestamps")
    }
}

#[cfg(test)]
mod tests {
    use super::{DISCORD_EPOCH, DiscordIds, MAX_DISCORD_ID_MILLIS, Timestamp, TimestampOutOfRange};

    #[test]
    fn test_try_as_discord_id() {
        // https://docs.discord.com/developers/reference#convert-snowflake-to-datetime
        let timestamp = Timestamp::from_millisecond(1462015105796).unwrap();
        let as_discord_id = timestamp.try_as_discord_id().unwrap();
        assert_eq!(as_discord_id, 175928847298985984);

        let too_early = Timestamp::from_millisecond((DISCORD_EPOCH - 1) as i64).unwrap();
        let result = too_early.try_as_discord_id();
        assert!(matches!(result, Err(TimestampOutOfRange)));

        let too_late = Timestamp::from_millisecond((MAX_DISCORD_ID_MILLIS + 1) as i64).unwrap();
        let result = too_late.try_as_discord_id();
        assert!(matches!(result, Err(TimestampOutOfRange)));

        // The Discord epoch itself should be a valid conversion, and should produce an id of 0.
        let just_right = Timestamp::from_millisecond(DISCORD_EPOCH as i64).unwrap();
        let zero = just_right.try_as_discord_id().unwrap();
        assert_eq!(zero, 0);
    }

    #[test]
    fn test_from_discord_id() {
        // https://docs.discord.com/developers/reference#convert-snowflake-to-datetime
        let timestamp = Timestamp::from_millisecond(1462015105796).unwrap();
        let from_discord_id = Timestamp::from_discord_id(175928847298985984);
        assert_eq!(timestamp, from_discord_id);

        // The Discord epoch itself should be a valid conversion, and should be produced from an id
        // of 0.
        let just_right = Timestamp::from_millisecond(DISCORD_EPOCH as i64).unwrap();
        let zero = Timestamp::from_discord_id(0);
        assert_eq!(just_right, zero);
    }
}
