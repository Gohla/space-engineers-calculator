use std::collections::HashMap;
use std::path::{Path, PathBuf};

use roxmltree::Document;
use snafu::{Backtrace, OptionExt, ResultExt, Snafu};

use crate::error::ErrorExt;

use super::localization::Localization;
use super::xml::{NodeExt, read_string_from_file};

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("Could not read components file '{}': {}", file_path.display(), source))]
  ReadFile { file_path: PathBuf, source: std::io::Error, },
  #[snafu(display("Could not XML parse components file '{}': {}", file_path.display(), source))]
  ParseFile { file_path: PathBuf, source: roxmltree::Error, },
  #[snafu(display("Unexpected XML structure"))]
  XmlStructure { backtrace: Backtrace },
  #[snafu(display("Could not parse text of XML element: {}", source))]
  XmlParseText { source: Box<dyn std::error::Error>, backtrace: Backtrace },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;


#[derive(Clone, Debug)]
pub struct Component {
  pub name: String,
  pub mass: f64,
  pub volume: f64,
}

impl Component {
  pub fn name<'a>(&'a self, localization: &'a Localization) -> &'a str {
    localization.get(&self.name).unwrap_or(&self.name)
  }
}


#[derive(Clone, Debug)]
pub struct Components {
  pub components: HashMap<String, Component>,
}

impl Components {
  pub fn from_se_dir<P: AsRef<Path>>(se_dir_path: P) -> Result<Self> {
    Self::from_sbc_file(se_dir_path.as_ref().join("Content/Data/Components.sbc"))
  }

  pub fn from_sbc_file<P: AsRef<Path>>(file_path: P) -> Result<Self> {
    let file_path = file_path.as_ref();
    let string = read_string_from_file(file_path).context(self::ReadFile { file_path })?;
    let doc = Document::parse(&string).context(self::ParseFile { file_path })?;

    let mut components = HashMap::new();

    let root_element = doc.root()
      .first_element_child().context(self::XmlStructure)?
      .first_element_child().context(self::XmlStructure)?;
    for component in root_element.children_elems("Component") {
      let id_node = component.child_elem("Id").context(self::XmlStructure)?;
      let id = id_node.parse_child_elem("SubtypeId").unwrap() /* CORRECTNESS: String FromStr never fails. */
        .context(self::XmlStructure)?;
      let name = component.parse_child_elem("DisplayName").unwrap() /* CORRECTNESS: String FromStr never fails. */
        .context(self::XmlStructure)?;
      let mass = component.parse_child_elem::<f64>("Mass")
        .map_err(|e| e.into_boxed())
        .context(self::XmlParseText)?
        .context(self::XmlStructure)?;
      let volume = component.parse_child_elem::<f64>("Volume")
        .map_err(|e| e.into_boxed())
        .context(self::XmlParseText)?
        .context(self::XmlStructure)?;
      components.insert(id, Component { name, mass, volume });
    }

    Ok(Self { components })
  }

  pub fn get(&self, id: &str) -> Option<&Component> {
    self.components.get(id)
  }
}
