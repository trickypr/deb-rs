#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::shared::PackageWithVersion;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PathItem {
    pub real: String,
    pub move_to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Version {
    V1_0,
    V2_0,
    VUnknown,
}

/**
 * Type doc: <https://www.debian.org/doc/debian-policy/ch-controlfields.html#s-binarycontrolfiles>
 * YAML format
*/
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Control {
    pub package: String,
    pub source: Option<String>,
    pub version: String,
    pub section: Option<String>,
    pub priority: Option<String>,
    pub architecture: String,
    pub essential: Option<String>,
    pub install_size: Option<u64>, // There could be a better value for this, however rust-yaml outputs it as i64
    pub maintainer: String,
    pub description: String,
    pub homepage: Option<String>,
    pub built_using: Option<String>,
    // Depends et al: <https://www.debian.org/doc/debian-policy/ch-relationships.html#s-binarydeps>
    pub depends: Vec<PackageWithVersion>,
    pub pre_depends: Vec<PackageWithVersion>,
    pub recommends: Vec<PackageWithVersion>,
    pub suggests: Vec<PackageWithVersion>,
    pub enhances: Vec<PackageWithVersion>,
    pub breaks: Vec<PackageWithVersion>,
    pub conflicts: Vec<PackageWithVersion>,
}
