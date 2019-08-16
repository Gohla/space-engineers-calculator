use std::collections::HashMap;
use std::path::Path;

use roxmltree::Document;

use super::xml::{NodeExt, read_string_from_file};

#[derive(Clone, Debug)]
pub struct Localization {
  pub localization: HashMap<String, String>,
}

impl Localization {
  pub fn from_se_dir<P: AsRef<Path>>(se_dir: P) -> Self {
    Self::from_resx_file(se_dir.as_ref().join("Content/Data/Localization/MyTexts.resx"))
  }

  pub fn from_resx_file<P: AsRef<Path>>(resx_path: P) -> Self {
    let string = read_string_from_file(resx_path).unwrap();
    let doc = Document::parse(&string).unwrap();

    let mut localization = HashMap::new();

    for node in doc.root().first_element_child().unwrap().children_elems("data") {
      if let Some(name) = node.attribute("name") {
        if let Some(value_node) = node.first_element_child() {
          if let Some(value) = value_node.text() {
            localization.insert(name.to_string(), value.to_string());
          }
        }
      }
    }

    Self { localization }
  }


  pub fn get(&self, id: &str) -> Option<&String> {
    self.localization.get(id)
  }
}
