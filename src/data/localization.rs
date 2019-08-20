use std::collections::HashMap;
use std::path::{Path, PathBuf};

use roxmltree::Document;
use snafu::{Backtrace, OptionExt, ResultExt, Snafu};

use super::xml::{NodeExt, read_string_from_file};

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("Could not read localization file '{}': {}", file_path.display(), source))]
  ReadFile { file_path: PathBuf, source: std::io::Error, },
  #[snafu(display("Could not XML parse localization file '{}': {}", file_path.display(), source))]
  ParseFile { file_path: PathBuf, source: roxmltree::Error, },
  #[snafu(display("Unexpected XML structure"))]
  XmlStructure { backtrace: Backtrace },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;


#[derive(Clone, Debug)]
pub struct Localization {
  pub localization: HashMap<String, String>,
}

impl Localization {
  pub fn from_se_dir<P: AsRef<Path>>(se_dir_path: P) -> Result<Self> {
    Self::from_resx_file(se_dir_path.as_ref().join("Content/Data/Localization/MyTexts.resx"))
  }

  pub fn from_resx_file<P: AsRef<Path>>(file_path: P) -> Result<Self> {
    let file_path = file_path.as_ref();
    let string = read_string_from_file(file_path).context(self::ReadFile { file_path })?;
    let doc = Document::parse(&string).context(self::ParseFile { file_path })?;

    let mut localization = HashMap::new();

    let root_element = doc.root().first_element_child().context(self::XmlStructure {})?;
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


  pub fn get(&self, id: &str) -> Option<&String> {
    self.localization.get(id)
  }
}
