use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

use roxmltree::Node;

pub fn read_string_from_file<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
  let mut file = File::open(path)?;
  let mut buf = String::new();
  file.read_to_string(&mut buf)?;
  Ok(buf)
}

pub trait NodeExt {
  fn get_child_elem(&self, tag: &'static str) -> Option<Node>;
  fn parse_child_elem<T: FromStr>(&self, tag: &'static str) -> Result<Option<T>, T::Err>;
}

impl<'a, 'input: 'a> NodeExt for Node<'a, 'input> {
  fn get_child_elem(&self, tag: &'static str) -> Option<Node> {
    for node in self.children() {
      if !node.is_element() { continue }
      if !node.has_tag_name(tag) { continue }
      return Some(node);
    }
    None
  }

  fn parse_child_elem<T: FromStr>(&self, tag: &'static str) -> Result<Option<T>, T::Err> {
    for node in self.children() {
      if !node.is_element() { continue }
      if !node.has_tag_name(tag) { continue }
      if let Some(text) = node.text() {
        return text.parse().map(|v| Some(v));
      }
    }
    Ok(None)
  }
}
