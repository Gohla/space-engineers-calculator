use std::collections::HashMap;
use std::path::Path;

use roxmltree::Document;

use super::localization::Localization;
use super::xml::{NodeExt, read_string_from_file};

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
  pub fn from_se_dir<P: AsRef<Path>>(se_dir: P) -> Self {
    Self::from_sbc_file(se_dir.as_ref().join("Content/Data/Components.sbc"))
  }

  pub fn from_sbc_file<P: AsRef<Path>>(components_sbc_path: P) -> Self {
    let string = read_string_from_file(components_sbc_path).unwrap();
    let doc = Document::parse(&string).unwrap();

    let mut components = HashMap::new();

    for component in doc.root().first_element_child().unwrap().first_element_child().unwrap().children_elems("Component") {
      let id = component.child_elem("Id").unwrap().parse_child_elem("SubtypeId").unwrap().unwrap();
      let name = component.parse_child_elem("DisplayName").unwrap().unwrap();
      let mass = component.parse_child_elem("Mass").unwrap().unwrap();
      let volume = component.parse_child_elem("Volume").unwrap().unwrap();
      components.insert(id, Component { name, mass, volume });
    }

    Self { components }
  }

  pub fn get(&self, id: &str) -> Option<&Component> {
    self.components.get(id)
  }
}
