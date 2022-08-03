use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use blocks::Blocks;
use components::Components;
use gas_properties::GasProperties;
use localization::Localization;

pub mod blocks;
pub mod components;
pub mod gas_properties;
pub mod localization;
pub mod xml;

#[derive(Error, Debug)]
pub enum ExtractError {
  #[error("Could not read blocks")]
  ReadBlocks(#[from] blocks::extract::Error),
  #[error("Could not read components")]
  ReadComponents(#[from] components::Error),
  #[error("Could not read gas properties")]
  ReadGasProperties(#[from] gas_properties::Error),
  #[error("Could not read localization")]
  ReadLocalization(#[from] localization::Error),
}

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

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Data {
  pub blocks: Blocks,
  pub components: Components,
  pub gas_properties: GasProperties,
  pub localization: Localization,
}

impl Data {
  pub fn extract_from_se_dir<P: AsRef<Path>>(se_dir_path: P) -> Result<Self, ExtractError> {
    let se_dir_path = se_dir_path.as_ref();
    let localization = Localization::from_se_dir(se_dir_path)?;
    let blocks = Blocks::from_se_dir(se_dir_path, &localization)?;
    let components = Components::from_se_dir(se_dir_path)?;
    let gas_properties = GasProperties::from_se_dir(se_dir_path)?;
    Ok(Self { blocks, components, gas_properties, localization })
  }

  pub fn from_json<R: io::Read>(reader: R) -> Result<Self, ReadError> {
    let data = serde_json::from_reader(reader)?;
    Ok(data)
  }

  pub fn to_json<W: io::Write>(&self, writer: W) -> Result<(), WriteError> {
    serde_json::to_writer_pretty(writer, self)?;
    Ok(())
  }
}
