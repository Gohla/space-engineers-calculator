use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Mods {
  pub mods: LinkedHashMap<u64, Mod>,
}

impl Mods {
  #[inline]
  pub fn new(mods: impl Iterator<Item=Mod>) -> Self {
    let mut map = LinkedHashMap::new();
    for m in mods {
      map.insert(m.0, m);
    }
    Self { mods: map }
  }

  #[inline]
  pub fn get(&self, id: &u64) -> Option<&Mod> { self.mods.get(id) }
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct Mod(pub u64, pub String);
