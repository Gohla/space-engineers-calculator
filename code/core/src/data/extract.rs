use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::data::{blocks, components, Data, gas_properties, localization};
use crate::data::blocks::extract::BlocksBuilder;
use crate::data::components::Components;
use crate::data::gas_properties::GasProperties;
use crate::data::localization::extract::LocalizationBuilder;
use crate::data::mods::{Mod, Mods};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ExtractConfig {
  pub extract_mods: Vec<Mod>,

  pub hide_block_by_exact_name: Vec<String>,
  pub hide_block_by_regex_name: Vec<String>,
  pub hide_block_by_exact_subtype_id: Vec<String>,
  pub hide_block_by_regex_subtype_id: Vec<String>,
  pub hide_block_by_exact_id: Vec<String>,
  pub hide_block_by_regex_id: Vec<String>,
  pub rename_block_by_regex: Vec<(String, String)>,
}

#[derive(Error, Debug)]
pub enum ExtractError {
  #[error("Could not create blocks builder")]
  CreateBlocksBuilderFail {
    #[from]
    source: blocks::extract::CreateError
  },
  #[error("Could not extract blocks")]
  ExtractBlocksFail {
    #[from]
    source: blocks::extract::ExtractError
  },
  #[error("Could not extract components")]
  ExtractComponentsFail {
    #[from]
    source: components::extract::Error
  },
  #[error("Could not extract gas properties")]
  ExtractGasPropertiesFail {
    #[from]
    source: gas_properties::extract::Error
  },
  #[error("Could not extract localization")]
  ExtractLocalizationFail {
    #[from]
    source: localization::extract::Error
  },
}

impl Data {
  pub fn extract_from_se_dir(
    se_directory: impl AsRef<Path>,
    se_workshop_directory: Option<impl AsRef<Path>>,
    extract_config: ExtractConfig,
  ) -> Result<Self, ExtractError> {
    let se_directory = se_directory.as_ref();
    // Mods
    let mods = Mods::new(extract_config.extract_mods.into_iter());
    // Localization
    let mut localization_builder = LocalizationBuilder::default();
    localization_builder.update_from_se_dir(se_directory)?;
    if let Some(se_workshop_directory) = &se_workshop_directory {
      for mod_id in mods.mods.keys() {
        localization_builder.update_from_mod(&se_workshop_directory, *mod_id)?;
      }
    }
    let localization = localization_builder.into_localization();
    // Blocks
    let mut blocks_builder = BlocksBuilder::new(
      extract_config.hide_block_by_exact_name.into_iter(),
      extract_config.hide_block_by_regex_name.into_iter(),
      extract_config.hide_block_by_exact_subtype_id.into_iter(),
      extract_config.hide_block_by_regex_subtype_id.into_iter(),
      extract_config.hide_block_by_exact_id.into_iter(),
      extract_config.hide_block_by_regex_id.into_iter(),
      extract_config.rename_block_by_regex.into_iter(),
    )?;
    blocks_builder.update_from_se_dir(se_directory, &localization)?;
    if let Some(se_workshop_directory) = &se_workshop_directory {
      for mod_id in mods.mods.keys() {
        blocks_builder.update_from_mod(se_directory, &se_workshop_directory, *mod_id, &localization)?;
      }
    }
    let blocks = blocks_builder.into_blocks(&localization);
    // Components
    let components = Components::from_se_dir(se_directory)?;
    // Gas properties
    let gas_properties = GasProperties::from_se_dir(se_directory)?;
    // Data
    Ok(Self { blocks, components, gas_properties, localization, mods })
  }
}
