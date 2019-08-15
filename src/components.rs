use std::collections::HashMap;
use std::path::Path;

use roxmltree::Document;

use crate::xml::{NodeExt, read_string_from_file};

#[derive(Clone, Debug)]
pub struct Component {
  pub name: String,
  pub mass: f64,
  pub volume: f64,
}

#[derive(Clone, Debug)]
pub struct Components {
  pub components: HashMap<String, Component>,
}

impl Components {
  pub fn from_data<P: AsRef<Path>>(components_sbc_path: P) -> Components {
    let string = read_string_from_file(components_sbc_path).unwrap();
    let doc = Document::parse(&string).unwrap();

    let mut components = HashMap::new();

    for component in doc.root().first_element_child().unwrap().first_element_child().unwrap().children() {
      if !component.is_element() { continue }
      let id = component.get_child_elem("Id").unwrap().parse_child_elem("SubtypeId").unwrap().unwrap();
      let name = component.parse_child_elem("DisplayName").unwrap().unwrap();
      let mass = component.parse_child_elem("Mass").unwrap().unwrap();
      let volume = component.parse_child_elem("Volume").unwrap().unwrap();
      components.insert(id, Component { name, mass, volume });
    }

    Components { components }
  }

  pub fn get(&self, id: &str) -> Option<&Component> {
    self.components.get(id)
  }
}
