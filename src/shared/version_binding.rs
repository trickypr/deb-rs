#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum VersionBinding {
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Equal,
    Any,
    Unknown,
}

impl VersionBinding {
    pub fn from_str(s: &str) -> Self {
        let s = s.split(' ').collect::<Vec<&str>>()[0];

        match s {
            ">" => VersionBinding::GreaterThan,
            "<" => VersionBinding::LessThan,
            ">=" => VersionBinding::GreaterThanOrEqual,
            "<=" => VersionBinding::LessThanOrEqual,
            "=" => VersionBinding::Equal,
            _ => VersionBinding::Unknown,
        }
    }
}
