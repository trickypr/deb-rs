#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::VersionBinding;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PackageWithVersion {
    pub name: String,
    pub version: String,
    pub binding: VersionBinding,
}

impl PackageWithVersion {
    pub fn from_str(contents: &str) -> Self {
        let split: Vec<&str> = contents.split(')').collect::<Vec<&str>>()[0]
            .split(" (")
            .collect();

        if split.len() == 2 {
            let name = split[0].to_string();
            let mut version = split[1].to_string();
            let mut version_binding = String::new();
            loop {
                let first = version.chars().next().unwrap();

                if !(first == '=' || first == '>' || first == '<' || first == ' ') {
                    break;
                } else {
                    version_binding.push(first);
                    version.remove(0);
                }
            }

            PackageWithVersion {
                name,
                version,
                binding: VersionBinding::from_str(&version_binding),
            }
        } else {
            let name = split[0].to_string();

            PackageWithVersion {
                name,
                version: String::new(),
                binding: VersionBinding::Any,
            }
        }
    }
}
