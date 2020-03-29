use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

use roxmltree::{Children, ExpandedName, Node};

pub fn read_string_from_file<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
  let mut file = File::open(path)?;
  let mut buf = String::new();
  file.read_to_string(&mut buf)?;
  Ok(buf)
}

pub trait NodeExt<'a, 'input: 'a> {
  fn child_elem(&self, tag: &'static str) -> Option<Node>;
  fn children_elems(&self, tag: &'static str) -> ElemChildren;
  fn parse_child_elem<T: FromStr>(&self, tag: &'static str) -> Result<Option<T>, T::Err>;
  fn parse_attribute<T: FromStr, N: Into<ExpandedName<'a>>>(&self, name: N) -> Result<Option<T>, T::Err>;
}

impl<'a, 'input: 'a> NodeExt<'a, 'input> for Node<'a, 'input> {
  fn child_elem(&self, tag: &'static str) -> Option<Node> {
    for node in self.children() {
      if !node.is_element() { continue }
      if !node.has_tag_name(tag) { continue }
      return Some(node);
    }
    None
  }

  fn children_elems(&self, tag: &'static str) -> ElemChildren {
    ElemChildren { children: self.children(), tag }
  }

  fn parse_child_elem<T: FromStr>(&self, tag: &'static str) -> Result<Option<T>, T::Err> {
    for node in self.children() {
      if !node.is_element() { continue }
      if !node.has_tag_name(tag) { continue }
      if let Some(text) = node.text() {
        return text.trim().parse().map(|v| Some(v));
      }
    }
    Ok(None)
  }

  fn parse_attribute<T: FromStr, N: Into<ExpandedName<'a>>>(&self, name: N) -> Result<Option<T>, T::Err> {
    if let Some(attribute) = self.attribute(name) {
      return attribute.trim().parse().map(|v| Some(v))
    }
    Ok(None)
  }
}

#[derive(Clone)]
pub struct ElemChildren<'a, 'input: 'a> {
  children: Children<'a, 'input>,
  tag: &'static str,
}

impl<'a, 'input: 'a> Iterator for ElemChildren<'a, 'input> {
  type Item = Node<'a, 'input>;

  fn next(&mut self) -> Option<Self::Item> {
    let mut elem = self.children.next();
    while let Some(node) = elem {
      if !node.is_element() || !node.has_tag_name(self.tag) {
        elem = self.children.next();
      } else {
        return Some(node)
      }
    }
    None
  }
}