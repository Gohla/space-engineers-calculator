use std::collections::HashMap;
use std::path::{Path, PathBuf};

use roxmltree::Document;
use serde::{Deserialize, Serialize};
use snafu::{Backtrace, OptionExt, ResultExt, Snafu};

use crate::error::ErrorExt;

use super::localization::Localization;
use super::xml::{NodeExt, read_string_from_file};

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("Could not read localization file '{}': {}", file_path.display(), source))]
  ReadFile { file_path: PathBuf, source: std::io::Error, },
  #[snafu(display("Could not XML parse localization file '{}': {}", file_path.display(), source))]
  ParseFile { file_path: PathBuf, source: roxmltree::Error, },
  #[snafu(display("Unexpected XML structure"))]
  XmlStructure { backtrace: Backtrace },
  #[snafu(display("Could not parse text of XML element: {}", source))]
  XmlParseText { source: Box<dyn std::error::Error>, backtrace: Backtrace },
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
    let string = read_string_from_file(file_path).context(self::ReadFile { file_path })?;
    let doc = Document::parse(&string).context(self::ParseFile { file_path })?;

    let mut gas_properties = HashMap::new();

    let root_element = doc.root()
      .first_element_child().context(self::XmlStructure)?
      .first_element_child().context(self::XmlStructure)?;
    for gas in root_element.children_elems("Gas") {
      let id_node = gas.child_elem("Id").context(self::XmlStructure)?;
      let id: String = id_node.parse_child_elem("SubtypeId").unwrap() /* CORRECTNESS: String FromStr never fails. */
        .context(self::XmlStructure)?;
      let name = id.clone();
      let energy_density = gas.parse_child_elem::<f64>("EnergyDensity")
        .map_err(|e| e.into_boxed())
        .context(self::XmlParseText)?
        .unwrap_or(0.0);
      gas_properties.insert(id, GasProperty { name, energy_density });
    }

    Ok(GasProperties { gas_properties })
  }


  pub fn get(&self, id: &str) -> Option<&GasProperty> {
    self.gas_properties.get(id)
  }
}
