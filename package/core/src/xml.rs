use std::backtrace::Backtrace;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::num::ParseFloatError;
use std::path::Path;
use std::str::{FromStr, ParseBoolError};

use roxmltree::{Children, ExpandedName, Node};
use thiserror::Error;

use crate::error::ErrorExt;

// XML errors

/// Type alias for [`Backtrace`], ensuring `thiserror` does not use nightly features.
#[cfg(not(nightly))]
pub type BT = Backtrace;

#[derive(Error, Debug)]
pub enum XmlError {
  #[cfg(nightly)]
  #[error("Unexpected XML structure")]
  StructureFail(Backtrace),
  #[cfg(not(nightly))]
  #[error("Unexpected XML structure")]
  StructureFail(BT),
  #[cfg(nightly)]
  #[error("Could not parse text or attribute of an XML element")]
  ParseTextFail(#[from] Box<dyn std::error::Error + 'static + Send + Sync>, Backtrace),
  #[cfg(not(nightly))]
  #[error("Could not parse text or attribute of an XML element")]
  ParseTextFail(#[source] Box<dyn std::error::Error + 'static + Send + Sync>, BT),
}

impl From<ParseFloatError> for XmlError {
  fn from(e: ParseFloatError) -> Self {
    Self::ParseTextFail(e.into_boxed(),  Backtrace::capture())
  }
}

impl From<ParseBoolError> for XmlError {
  fn from(e: ParseBoolError) -> Self {
    Self::ParseTextFail(e.into_boxed(),  Backtrace::capture())
  }
}

// XML convenience extension

pub trait NodeExt<'a, 'input: 'a> {
  fn child_elem(&self, tag: &'static str) -> Result<Node, XmlError>;
  fn child_elem_opt(&self, tag: &'static str) -> Option<Node>;
  fn first_child_elem(&self) -> Result<Node, XmlError>;
  fn children_elems(&self, tag: &'static str) -> ElemChildren;

  fn text_or_err(&self) -> Result<&str, XmlError>;

  fn parse_child_elem<T: FromStr>(&self, tag: &'static str) -> Result<T, XmlError> where T::Err: Error + Send + Sync + 'static;
  fn parse_child_elem_opt<T: FromStr>(&self, tag: &'static str) -> Result<Option<T>, XmlError> where T::Err: Error + Send + Sync + 'static;

  fn parse_attribute<T: FromStr, N: Into<ExpandedName<'a, 'a>>>(&self, name: N) -> Result<T, XmlError> where T::Err: Error + Send + Sync + 'static;
}

impl<'a, 'input: 'a> NodeExt<'a, 'input> for Node<'a, 'input> {
  fn child_elem(&self, tag: &'static str) -> Result<Node, XmlError> {
    for node in self.children() {
      if !node.is_element() { continue }
      if !node.has_tag_name(tag) { continue }
      return Ok(node);
    }
    Err(XmlError::StructureFail(Backtrace::capture()))
  }
  fn child_elem_opt(&self, tag: &'static str) -> Option<Node> {
    for node in self.children() {
      if !node.is_element() { continue }
      if !node.has_tag_name(tag) { continue }
      return Some(node);
    }
    None
  }
  fn first_child_elem(&self) -> Result<Node, XmlError> {
    self.first_element_child()
      .ok_or_else(|| XmlError::StructureFail(Backtrace::capture()))
  }
  fn children_elems(&self, tag: &'static str) -> ElemChildren {
    ElemChildren { children: self.children(), tag }
  }


  fn text_or_err(&self) -> Result<&str, XmlError> {
    self.text()
      .ok_or_else(|| XmlError::StructureFail(Backtrace::capture()))
  }


  fn parse_child_elem<T: FromStr>(&self, tag: &'static str) -> Result<T, XmlError> where T::Err: Error + Send + Sync + 'static {
    for node in self.children() {
      if !node.is_element() { continue }
      if !node.has_tag_name(tag) { continue }
      if let Some(text) = node.text() {
        return text.trim().parse()
          .map_err(|e: <T as FromStr>::Err| XmlError::ParseTextFail(e.into_boxed(), Backtrace::capture()));
      }
    }
    Err(XmlError::StructureFail(Backtrace::capture()))
  }
  fn parse_child_elem_opt<T: FromStr>(&self, tag: &'static str) -> Result<Option<T>, XmlError> where T::Err: Error + Send + Sync + 'static {
    for node in self.children() {
      if !node.is_element() { continue }
      if !node.has_tag_name(tag) { continue }
      if let Some(text) = node.text() {
        return text.trim().parse()
          .map(|v| Some(v))
          .map_err(|e: <T as FromStr>::Err| XmlError::ParseTextFail(e.into_boxed(), Backtrace::capture()));
      }
    }
    Ok(None)
  }


  fn parse_attribute<T: FromStr, N: Into<ExpandedName<'a, 'a>>>(&self, name: N) -> Result<T, XmlError> where T::Err: Error + Send + Sync + 'static {
    if let Some(attribute) = self.attribute(name) {
      return attribute.trim().parse()
        .map_err(|e: <T as FromStr>::Err| XmlError::ParseTextFail(e.into_boxed(), Backtrace::capture()));
    }
    Err(XmlError::StructureFail(Backtrace::capture()))
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

// File reading convenience

pub fn read_string_from_file<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
  let mut file = File::open(path)?;
  let mut buf = String::new();
  file.read_to_string(&mut buf)?;
  Ok(buf)
}
