use std::collections::HashMap;
use std::path::Path;

use roxmltree::Document;

use super::localization::Localization;
use super::xml::{NodeExt, read_string_from_file};

#[derive(Clone, Debug)]
pub struct GasProperty {
  pub name: String,
  pub energy_density: f64,
}

impl GasProperty {
  pub fn name(&self, _localization: &Localization) -> &str {
    &self.name
  }
}


#[derive(Clone, Debug)]
pub struct GasProperties {
  pub gas_properties: HashMap<String, GasProperty>,
}

impl GasProperties {
  pub fn from_se_dir<P: AsRef<Path>>(se_dir: P) -> Self {
    Self::from_sbc_file(se_dir.as_ref().join("Content/Data/GasProperties.sbc"))
  }

  pub fn from_sbc_file<P: AsRef<Path>>(gasproperties_sbc_path: P) -> Self {
    let string = read_string_from_file(gasproperties_sbc_path).unwrap();
    let doc = Document::parse(&string).unwrap();

    let mut gas_properties = HashMap::new();

    for gas in doc.root().first_element_child().unwrap().first_element_child().unwrap().children_elems("Gas") {
      let id: String = gas.child_elem("Id").unwrap()
        .parse_child_elem("SubtypeId").unwrap().unwrap();
      let name = id.clone();
      let energy_density = gas.parse_child_elem("EnergyDensity").unwrap().unwrap_or(0.0);
      gas_properties.insert(id, GasProperty { name, energy_density });
    }

    GasProperties { gas_properties }
  }


  pub fn get(&self, id: &str) -> Option<&GasProperty> {
    self.gas_properties.get(id)
  }
}
