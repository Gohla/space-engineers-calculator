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
    localization.get(&self.name).unwrap_or(&self.name)
  }
}


// Extraction

#[cfg(feature = "extract")]
pub mod extract {
  use std::backtrace::Backtrace;
  use std::path::{Path, PathBuf};

  use linked_hash_map::LinkedHashMap;
  use roxmltree::Document;
  use thiserror::Error;

  use crate::data::components::{Component, Components};
  use crate::data::xml::{NodeExt, read_string_from_file};
  use crate::error::ErrorExt;

  #[derive(Error, Debug)]
  pub enum Error {
    #[error("Could not read components file '{file}'")]
    ReadFile { file: PathBuf, source: std::io::Error, },
    #[error("Could not XML parse components file '{file}'")]
    ParseFile { file: PathBuf, source: roxmltree::Error, },
    #[error("Unexpected XML structure")]
    XmlStructure(Backtrace),
    #[error("Could not parse text of XML element")]
    XmlParseText(#[from] Box<dyn std::error::Error>, Backtrace),
  }

  impl Components {
    pub fn from_se_dir<P: AsRef<Path>>(se_dir_path: P) -> Result<Self, Error> {
      Self::from_sbc_file(se_dir_path.as_ref().join("Content/Data/Components.sbc"))
    }

    pub fn from_sbc_file<P: AsRef<Path>>(file_path: P) -> Result<Self, Error> {
      let file_path = file_path.as_ref();
      let string = read_string_from_file(file_path)
        .map_err(|source| Error::ReadFile { file: file_path.to_path_buf(), source })?;
      let doc = Document::parse(&string)
        .map_err(|source| Error::ParseFile { file: file_path.to_path_buf(), source })?;

      let mut components = LinkedHashMap::new();

      let root_element = doc.root()
        .first_element_child().ok_or(Error::XmlStructure(Backtrace::capture()))?
        .first_element_child().ok_or(Error::XmlStructure(Backtrace::capture()))?;
      for component in root_element.children_elems("Component") {
        let id_node = component.child_elem("Id").ok_or(Error::XmlStructure(Backtrace::capture()))?;
        let id = id_node.parse_child_elem("SubtypeId").unwrap() /* CORRECTNESS: String FromStr never fails. */
          .ok_or(Error::XmlStructure(Backtrace::capture()))?;
        let name = component.parse_child_elem("DisplayName").unwrap() /* CORRECTNESS: String FromStr never fails. */
          .ok_or(Error::XmlStructure(Backtrace::capture()))?;
        let mass = component.parse_child_elem::<f64>("Mass")
          .map_err(|e| e.into_boxed())?
          .ok_or(Error::XmlStructure(Backtrace::capture()))?;
        let volume = component.parse_child_elem::<f64>("Volume")
          .map_err(|e| e.into_boxed())?
          .ok_or(Error::XmlStructure(Backtrace::capture()))?;
        components.insert(id, Component { name, mass, volume });
      }

      Ok(Self { components })
    }
  }
}