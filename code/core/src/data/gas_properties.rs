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
  use std::path::{Path, PathBuf};

  use linked_hash_map::LinkedHashMap;
  use roxmltree::Document;
  use thiserror::Error;

  use crate::data::gas_properties::{GasProperties, GasProperty};
  use crate::xml::{NodeExt, read_string_from_file, XmlError};

  #[derive(Error, Debug)]
  pub enum Error {
    #[error("Could not read localization file '{file}'")]
    ReadFile { file: PathBuf, source: std::io::Error, },
    #[error("Could not XML parse localization file '{file}'")]
    ParseFile { file: PathBuf, source: roxmltree::Error, },
    #[error(transparent)]
    XmlFail(#[from] XmlError),
  }

  impl GasProperties {
    pub fn from_se_dir<P: AsRef<Path>>(se_directory: P) -> Result<Self, Error> {
      Self::from_sbc_file(se_directory.as_ref().join("Content/Data/GasProperties.sbc"))
    }

    pub fn from_sbc_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
      let path = path.as_ref();
      let string = read_string_from_file(path)
        .map_err(|source| Error::ReadFile { file: path.to_path_buf(), source })?;
      let doc = Document::parse(&string)
        .map_err(|source| Error::ParseFile { file: path.to_path_buf(), source })?;

      let mut gas_properties = LinkedHashMap::new();

      let root_element = doc.root();
      let root_element = root_element.first_child_elem()?;
      let root_element = root_element.first_child_elem()?;
      for gas in root_element.children_elems("Gas") {
        let id_node = gas.child_elem("Id")?;
        let id: String = id_node.parse_child_elem("SubtypeId")?;
        let name = id.clone();
        let energy_density = gas.parse_child_elem_opt("EnergyDensity")?.unwrap_or_default();
        gas_properties.insert(id, GasProperty { name, energy_density });
      }

      Ok(GasProperties { gas_properties })
    }
  }
}