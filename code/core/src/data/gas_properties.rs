use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct GasProperties {
  pub gas_properties: LinkedHashMap<String, GasProperty>,
}

impl GasProperties {
  #[inline]
  pub fn get(&self, id: &str) -> Option<&GasProperty> { self.gas_properties.get(id) }
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct GasProperty {
  pub name: String,
  pub energy_density: f64,
}

impl GasProperty {
  #[inline]
  pub fn name(&self) -> &str { &self.name }
}


// Extraction

#[cfg(feature = "extract")]
pub mod extract {
  use std::backtrace::Backtrace;
  use std::path::{Path, PathBuf};

  use linked_hash_map::LinkedHashMap;
  use roxmltree::Document;
  use thiserror::Error;

  use crate::data::gas_properties::{GasProperties, GasProperty};
  use crate::data::xml::{NodeExt, read_string_from_file};
  use crate::error::ErrorExt;

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

  impl GasProperties {
    pub fn from_se_dir<P: AsRef<Path>>(se_dir_path: P) -> Result<Self, Error> {
      Self::from_sbc_file(se_dir_path.as_ref().join("Content/Data/GasProperties.sbc"))
    }

    pub fn from_sbc_file<P: AsRef<Path>>(file_path: P) -> Result<Self, Error> {
      let file_path = file_path.as_ref();
      let string = read_string_from_file(file_path)
        .map_err(|source| Error::ReadFile { file: file_path.to_path_buf(), source })?;
      let doc = Document::parse(&string)
        .map_err(|source| Error::ParseFile { file: file_path.to_path_buf(), source })?;

      let mut gas_properties = LinkedHashMap::new();

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
  }
}