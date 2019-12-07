use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use roxmltree::Document;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::error::ErrorExt;

use super::localization::Localization;
use super::xml::{NodeExt, read_string_from_file};

#[derive(Error, Debug)]
pub enum Error {
  #[error("Could not read localization file '{file}'")]
  ReadFile { file: PathBuf, source: std::io::Error, },
  #[error("Could not XML parse localization file '{file}'")]
  ParseFile { file: PathBuf, source: roxmltree::Error, },
  #[error("Unexpected XML structure")]
  XmlStructure(Backtrace),
  #[error("Could not parse text of XML element")]
  XmlParseText(#[from] Box<dyn std::error::Error>, Backtrace),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GasProperty {
  pub name: String,
  pub energy_density: f64,
}

impl GasProperty {
  pub fn name(&self, _localization: &Localization) -> &str {
    &self.name
  }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GasProperties {
  pub gas_properties: HashMap<String, GasProperty>,
}

impl GasProperties {
  pub fn from_se_dir<P: AsRef<Path>>(se_dir_path: P) -> Result<Self> {
    Self::from_sbc_file(se_dir_path.as_ref().join("Content/Data/GasProperties.sbc"))
  }

  pub fn from_sbc_file<P: AsRef<Path>>(file_path: P) -> Result<Self> {
    let file_path = file_path.as_ref();
    let string = read_string_from_file(file_path)
      .map_err(|source| Error::ReadFile { file: file_path.to_path_buf(), source })?;
    let doc = Document::parse(&string)
      .map_err(|source| Error::ParseFile { file: file_path.to_path_buf(), source })?;

    let mut gas_properties = HashMap::new();

    let root_element = doc.root()
      .first_element_child().ok_or(Error::XmlStructure(Backtrace::capture()))?
      .first_element_child().ok_or(Error::XmlStructure(Backtrace::capture()))?;
    for gas in root_element.children_elems("Gas") {
      let id_node = gas.child_elem("Id").ok_or(Error::XmlStructure(Backtrace::capture()))?;
      let id: String = id_node.parse_child_elem("SubtypeId").unwrap() /* CORRECTNESS: String FromStr never fails. */
        .ok_or(Error::XmlStructure(Backtrace::capture()))?;
      let name = id.clone();
      let energy_density = gas.parse_child_elem::<f64>("EnergyDensity")
        .map_err(|e| e.into_boxed())?
        .unwrap_or(0.0);
      gas_properties.insert(id, GasProperty { name, energy_density });
    }

    Ok(GasProperties { gas_properties })
  }


  pub fn get(&self, id: &str) -> Option<&GasProperty> {
    self.gas_properties.get(id)
  }
}
