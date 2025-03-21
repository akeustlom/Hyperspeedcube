use std::fmt;

use chrono::SubsecRound;
use serde::{Deserialize, Serialize};

/// Type used for UTC timestamps in log files.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Timestamp(chrono::DateTime<chrono::Utc>);
impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        write!(f, "{s}")
    }
}
impl std::str::FromStr for Timestamp {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> chrono::ParseResult<Self> {
        chrono::DateTime::from_str(s).map(Self)
    }
}
impl Timestamp {
    /// Returns the UTC timestamp for the present moment, according to the
    /// system clock.
    pub fn now() -> Self {
        Self(chrono::Utc::now().trunc_subsecs(3))
    }

    /// Returns the number of nanoseconds since the start of the current second.
    pub fn subsec_nanos(self) -> u32 {
        self.0.timestamp_subsec_nanos()
    }
}
