use std::io;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::blocks::Blocks;
use crate::data::components::Components;
use crate::data::gas_properties::GasProperties;
use crate::data::localization::Localization;
use crate::data::mods::Mods;

pub mod blocks;
pub mod components;
pub mod gas_properties;
pub mod localization;
pub mod mods;
#[cfg(feature = "extract")]
pub mod extract;

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Data {
  pub mods: Mods,
  pub localization: Localization,
  pub blocks: Blocks,
  pub components: Components,
  pub gas_properties: GasProperties,
}

// From/to JSON

#[derive(Error, Debug)]
pub enum ReadError {
  #[error("Could not read data from JSON")]
  FromJSON(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum WriteError {
  #[error("Could not write data to JSON")]
  ToJSON(#[from] serde_json::Error),
}

impl Data {
  pub fn from_json<R: io::Read>(reader: R) -> Result<Self, ReadError> {
    let data = serde_json::from_reader(reader)?;
    Ok(data)
  }

  pub fn to_json<W: io::Write>(&self, writer: W) -> Result<(), WriteError> {
    serde_json::to_writer_pretty(writer, self)?;
    Ok(())
  }
}
