use std::collections::HashMap;
use std::path::Path;

use roxmltree::Document;

use crate::xml::read_string_from_file;

#[derive(Clone, Debug)]
pub struct Localization {
  pub localization: HashMap<String, String>,
}

impl Localization {
  pub fn from_data<P: AsRef<Path>>(resx_path: P) -> Localization {
    let string = read_string_from_file(resx_path).unwrap();
    let doc = Document::parse(&string).unwrap();

    let mut localization = HashMap::new();

    for node in doc.root().first_element_child().unwrap().children() {
      if !node.is_element() { continue }
      if !node.has_tag_name("data") { continue }
      if let Some(name) = node.attribute("name") {
        if let Some(value_node) = node.first_element_child() {
          if let Some(value) = value_node.text() {
            localization.insert(name.to_string(), value.to_string());
          }
        }
      }
    }

    Localization { localization }
  }


  pub fn get(&self, id: &str) -> Option<&String> {
    self.localization.get(id)
  }
}
