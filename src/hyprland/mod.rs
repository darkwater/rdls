use std::path::PathBuf;

use anyhow::Context as _;
use serde::Deserialize;

pub mod commands;
pub mod dispatch;
pub mod events;

fn hyprland_rundir() -> anyhow::Result<PathBuf> {
    let uid = nix::unistd::Uid::current();
    let signature = std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE")
        .context("HYPRLAND_INSTANCE_SIGNATURE not set")?;

    Ok(PathBuf::from("/run/user")
        .join(uid.to_string())
        .join("hypr")
        .join(signature))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(transparent)]
pub struct WorkspaceId(pub i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct WindowAddress(#[serde(with = "window_address_serde")] pub u64);

mod window_address_serde {
    use serde::{self, Deserialize, Deserializer};

    // pub fn serialize<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    // where
    //     S: Serializer,
    // {
    //     serializer.serialize_str(&format!("0x{:x}", value))
    // }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let trimmed = s.trim_start_matches("0x");
        u64::from_str_radix(trimmed, 16).map_err(serde::de::Error::custom)
    }
}
