use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};

use super::localization::Localization;

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Components {
  pub components: LinkedHashMap<String, Component>,
}

impl Components {
  #[inline]
  pub fn get(&self, id: &str) -> Option<&Component> { self.components.get(id) }
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Component {
  pub name: String,
  pub mass: f64,
  pub volume: f64,
}

impl Component {
  #[inline]
  pub fn name<'a>(&'a self, localization: &'a Localization) -> &'a str {
    localization.get(&self.name)
  }
}


// Extraction

#[cfg(feature = "extract")]
pub mod extract {
  use std::path::{Path, PathBuf};

  use linked_hash_map::LinkedHashMap;
  use roxmltree::Document;
  use thiserror::Error;

  use crate::data::components::{Component, Components};
  use crate::xml::{NodeExt, read_string_from_file, XmlError};

  #[derive(Error, Debug)]
  pub enum Error {
    #[error("Could not read components file '{file}'")]
    ReadFileFail { file: PathBuf, source: std::io::Error, },
    #[error("Could not XML parse components file '{file}'")]
    ParseFileFail { file: PathBuf, source: roxmltree::Error, },
    #[error(transparent)]
    XmlFail {
      #[from]
      #[backtrace]
      source: XmlError
    },
  }

  impl Components {
    pub fn from_se_dir<P: AsRef<Path>>(se_directory: P) -> Result<Self, Error> {
      Self::from_sbc_file(se_directory.as_ref().join("Content/Data/Components.sbc"))
    }

    pub fn from_sbc_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
      let path = path.as_ref();
      let string = read_string_from_file(path)
        .map_err(|source| Error::ReadFileFail { file: path.to_path_buf(), source })?;
      let doc = Document::parse(&string)
        .map_err(|source| Error::ParseFileFail { file: path.to_path_buf(), source })?;

      let mut components = LinkedHashMap::new();

      let root_element = doc.root();
      let root_element = root_element.first_child_elem()?;
      let root_element = root_element.first_child_elem()?;
      for component in root_element.children_elems("Component") {
        let id_node = component.child_elem("Id")?;
        let id = id_node.parse_child_elem("SubtypeId")?;
        let name = component.parse_child_elem("DisplayName")?;
        let mass = component.parse_child_elem("Mass")?;
        let volume = component.parse_child_elem("Volume")?;
        components.insert(id, Component { name, mass, volume });
      }

      Ok(Self { components })
    }
  }
}