use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Localization {
  pub localization: LinkedHashMap<String, String>,
}

impl Localization {
  #[inline]
  pub fn get<'a>(&'a self, id: &'a str) -> &'a str {
    if let Some(name) = self.localization.get(id) {
      &name
    } else { // Some mods use {LOC:<name>} as DisplayName, remove those part and try again.
      let len = id.len();
      if len > 6 {
        if let Some(name) = self.localization.get(&id[5..len - 1]) {
          return name;
        }
      }
      id // Otherwise, just return the id as name.
    }
  }
}


// Extraction

#[cfg(feature = "extract")]
pub mod extract {
  use std::path::{Path, PathBuf};

  use linked_hash_map::LinkedHashMap;
  use roxmltree::Document;
  use thiserror::Error;
  use walkdir::WalkDir;

  use crate::data::localization::Localization;
  use crate::xml::{NodeExt, read_string_from_file, XmlError};

  #[derive(Default)]
  pub struct LocalizationBuilder {
    pub localization: LinkedHashMap<String, String>,
  }

  #[derive(Error, Debug)]
  pub enum Error {
    #[error("Could not read localization file '{file}'")]
    ReadFileFail { file: PathBuf, source: std::io::Error, },
    #[error("Could not XML parse localization file '{file}'")]
    ParseFileFail { file: PathBuf, source: roxmltree::Error, },
    #[error(transparent)]
    XmlFail {
      #[from]
      #[backtrace]
      source: XmlError
    },
  }

  impl LocalizationBuilder {
    pub fn update_from_se_dir(&mut self, se_directory: impl AsRef<Path>) -> Result<(), Error> {
      self.update_from_resx_file(se_directory.as_ref().join("Content/Data/Localization/MyTexts.resx"))
    }

    pub fn update_from_mod(
      &mut self,
      se_workshop_directory: impl AsRef<Path>,
      mod_id: u64,
    ) -> Result<(), Error> {
      let search_path = se_workshop_directory.as_ref().join(format!("{}", mod_id));
      let sbl_file_paths = WalkDir::new(&search_path)
        .into_iter()
        .filter_map(|de| {
          if let Ok(de) = de {
            let path = de.into_path();
            if !path.extension().map_or(false, |e| e == "sbl") { return None; }
            Some(path)
          } else {
            None
          }
        });
      let mut updated_localizations = false;
      for path in sbl_file_paths {
        updated_localizations |= self.update_from_sbl_file(path)?;
      }
      if !updated_localizations {
        // Try to look for MyTexts.resx file in case the mod has no .sbl files or no english or 
        // default localization in an .sbl file.
        let my_texts_resx_file_paths = WalkDir::new(&search_path)
          .into_iter()
          .filter_map(|de| {
            if let Ok(de) = de {
              let path = de.into_path();
              if !path.file_name().map_or(false, |n| n == "MyTexts.resx") { return None; }
              Some(path)
            } else {
              None
            }
          });
        for path in my_texts_resx_file_paths {
          self.update_from_resx_file(path)?;
        }
      }
      Ok(())
    }

    pub fn update_from_sbl_file(&mut self, path: impl AsRef<Path>) -> Result<bool, Error> {
      let path = path.as_ref();
      let string = read_string_from_file(path)
        .map_err(|source| Error::ReadFileFail { file: path.to_path_buf(), source })?;
      let doc = Document::parse(&string)
        .map_err(|source| Error::ParseFileFail { file: path.to_path_buf(), source })?;
      let root_element = doc.root();
      let root_element = root_element.first_child_elem()?;
      let resx_name: String = root_element.parse_child_elem("ResXName")?;
      let language: String = root_element.parse_child_elem("Language")?;
      let default: bool = root_element.parse_child_elem("Default")?;
      if language == "en-US" || default {
        let resx_path = path.parent().unwrap().join(resx_name); // Unwrap OK: path to file must have a parent directory.
        self.update_from_resx_file(resx_path)?;
        Ok(true)
      } else {
        Ok(false)
      }
    }

    pub fn update_from_resx_file(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
      let path = path.as_ref();
      let string = read_string_from_file(path)
        .map_err(|source| Error::ReadFileFail { file: path.to_path_buf(), source })?;
      let doc = Document::parse(&string)
        .map_err(|source| Error::ParseFileFail { file: path.to_path_buf(), source })?;
      let root_element = doc.root();
      let root_element = root_element.first_child_elem()?;
      for node in root_element.children_elems("data") {
        if let Some(name) = node.attribute("name") {
          if let Some(value_node) = node.first_element_child() {
            if let Some(value) = value_node.text() {
              self.localization.insert(name.to_string(), value.to_string());
            }
          }
        }
      }
      Ok(())
    }

    pub fn into_localization(self) -> Localization {
      Localization { localization: self.localization }
    }
  }
}