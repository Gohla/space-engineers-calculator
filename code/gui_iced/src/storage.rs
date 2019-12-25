use std::io;

use anyhow::{format_err, Result};
use linked_hash_map::LinkedHashMap;
use log::error;
use serde::{Deserialize, Serialize};

use secalc_core::grid::GridCalculator;

#[derive(Default, Serialize, Deserialize)]
pub struct Storage {
  pub calculator: GridCalculator,
  pub calculator_name: Option<String>,
  pub calculator_modified: bool,

  saved_calculators: LinkedHashMap<String, GridCalculator>
}

impl Storage {
  pub fn contains_saved_calculator(&self, key: &str) -> bool {
    self.saved_calculators.contains_key(key)
  }

  pub fn iter_saved_calculators(&self) -> impl Iterator<Item=(&String, &GridCalculator)> {
    self.saved_calculators.iter()
  }


  pub fn save_calculator(&mut self, name: String) -> Result<()> {
    self.calculator_name = Some(name.clone());
    self.calculator_modified = false;
    self.saved_calculators.insert(name, self.calculator.clone());
    self.save()
  }

  pub fn load_calculator(&mut self, name: String) -> Result<()> {
    if let Some(calculator) = self.saved_calculators.get(&name) {
      self.calculator = calculator.clone();
      self.calculator_name = Some(name);
      self.calculator_modified = false;
      self.save()
    } else {
      Err(format_err!("No saved calculator with name '{}' was found", name))
    }
  }


  pub fn save(&self) -> Result<()> {
    self.save_internal()
  }

  pub fn load() -> Result<Option<Storage>> {
    Self::load_internal()
  }


  pub fn from_json<R: io::Read>(reader: R) -> Result<Self> {
    Ok(serde_json::from_reader::<_, Self>(reader)?)
  }

  pub fn from_json_string(string: &str) -> Result<Self> {
    Ok(serde_json::from_str::<Self>(string)?)
  }

  pub fn to_json<W: io::Write>(&self, writer: W) -> Result<()> {
    Ok(serde_json::to_writer_pretty(writer, self)?)
  }

  pub fn to_json_string(&self) -> Result<String> {
    Ok(serde_json::to_string_pretty(self)?)
  }
}

impl Drop for Storage {
  fn drop(&mut self) {
    self.save()
      .unwrap_or_else(|e| error!("[BUG] Could not save storage: {}", e));
  }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
  use std::fs::OpenOptions;
  use std::path::PathBuf;

  use anyhow::{Context, format_err, Result};

  use super::Storage;

  impl Storage {
    pub(crate) fn save_internal(&self) -> Result<()> {
      let storage_file = Self::get_storage_file()?;
      let writer = OpenOptions::new().write(true).create(true).open(storage_file.clone())
        .with_context(|| format!("Failed to open file '{:?}' for writing", storage_file))?;
      Ok(self.to_json(writer)?)
    }

    pub(crate) fn load_internal() -> Result<Option<Storage>> {
      let storage_file = Self::get_storage_file()?;
      if !storage_file.exists() {
        Ok(None)
      } else {
        let reader = OpenOptions::new().read(true).open(storage_file.clone())
          .with_context(|| format!("Failed to open file '{:?}' for reading", storage_file))?;
        Ok(Some(Self::from_json(reader)?))
      }
    }

    const STORAGE_FILE: &'static str = "storage.json";

    fn get_storage_file() -> Result<PathBuf> {
      let storage_dir = Self::get_storage_dir()?;
      std::fs::create_dir_all(storage_dir.clone())
        .with_context(|| format!("Failed to create directory '{:?}'", storage_dir))?;
      Ok(storage_dir.join(Self::STORAGE_FILE))
    }

    const STORAGE_SUBDIR: &'static str = "SECalc";

    fn get_storage_dir() -> Result<PathBuf> {
      if let Some(dir) = dirs::config_dir() {
        return Ok(dir.join(Self::STORAGE_SUBDIR));
      }
      if let Ok(dir) = std::env::current_dir() {
        return Ok(dir);
      }
      if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
          return Ok(dir.into());
        }
      }
      if let Some(dir) = dirs::home_dir() {
        return Ok(dir.join(Self::STORAGE_SUBDIR));
      }
      return Err(format_err!("Failed to get storage directory"))
    }
  }
}

#[cfg(target_arch = "wasm32")]
mod web {
  use anyhow::{format_err, Result};
  use thiserror::Error;

  use super::Storage;

  #[derive(Error, Debug)]
  pub enum GetLocalStorageError {
    #[error("Could not get local storage; cannot get window")]
    NoWindow,
    #[error("Could not get local storage; window returned no storage")]
    NoLocalStorage,
    #[error("Cannot get local storage; an error occurred")]
    LocalStorageError(String)
  }

  const STORAGE_KEY: &'static str = "secalc_storage";

  impl Storage {
    pub(crate) fn save_internal(&self) -> Result<()> {
      let storage = Self::get_local_storage()?;
      let string = self.to_json_string()?;
      Ok(storage.set(STORAGE_KEY, &string).map_err(|e| format_err!("{:?}", e))?)
    }

    pub(crate) fn load_internal() -> Result<Option<Storage>> {
      let storage = Self::get_local_storage()?;
      if let Some(string) = storage.get(STORAGE_KEY).map_err(|e| format_err!("{:?}", e))? {
        Ok(Some(Self::from_json_string(&string)?))
      } else {
        Ok(None)
      }
    }

    fn get_local_storage() -> Result<web_sys::Storage, GetLocalStorageError> {
      web_sys::window()
        .ok_or(GetLocalStorageError::NoWindow)?
        .local_storage()
        // Format exception into string, as JsValue is not `Send` and thus cannot be converted into `dyn Error` by `anyhow` later.
        .map_err(|e| GetLocalStorageError::LocalStorageError(format!("{:?}", e)))?
        .ok_or(GetLocalStorageError::NoLocalStorage)
    }
  }
}