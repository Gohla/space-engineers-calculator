use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Localization {
  pub localization: LinkedHashMap<String, String>,
}

impl Localization {
  #[inline]
  pub fn get(&self, id: &str) -> Option<&String> { self.localization.get(id) }
}


// Extraction

#[cfg(feature = "extract")]
pub mod extract {
  use std::backtrace::Backtrace;
  use std::path::{Path, PathBuf};

  use linked_hash_map::LinkedHashMap;
  use roxmltree::Document;
  use thiserror::Error;

  use crate::data::localization::Localization;
  use crate::data::xml::{NodeExt, read_string_from_file};

  #[derive(Error, Debug)]
  pub enum Error {
    #[error("Could not read localization file '{file}'")]
    ReadFile { file: PathBuf, source: std::io::Error, },
    #[error("Could not XML parse localization file '{file}'")]
    ParseFile { file: PathBuf, source: roxmltree::Error, },
    #[error("Unexpected XML structure")]
    XmlStructure(Backtrace),
  }

  impl Localization {
    pub fn from_se_dir<P: AsRef<Path>>(se_dir_path: P) -> Result<Self, Error> {
      Self::from_resx_file(se_dir_path.as_ref().join("Content/Data/Localization/MyTexts.resx"))
    }

    pub fn from_resx_file<P: AsRef<Path>>(file_path: P) -> Result<Self, Error> {
      let file_path = file_path.as_ref();
      let string = read_string_from_file(file_path)
        .map_err(|source| Error::ReadFile { file: file_path.to_path_buf(), source })?;
      let doc = Document::parse(&string)
        .map_err(|source| Error::ParseFile { file: file_path.to_path_buf(), source })?;

      let mut localization = LinkedHashMap::new();

      let root_element = doc.root().first_element_child()
        .ok_or(Error::XmlStructure(Backtrace::capture()))?;
      for node in root_element.children_elems("data") {
        if let Some(name) = node.attribute("name") {
          if let Some(value_node) = node.first_element_child() {
            if let Some(value) = value_node.text() {
              localization.insert(name.to_string(), value.to_string());
            }
          }
        }
      }

      Ok(Self { localization })
    }
  }
}