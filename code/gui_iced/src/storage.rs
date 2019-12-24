use linked_hash_map::LinkedHashMap;

use secalc_core::grid::GridCalculator;

#[derive(Default)]
pub struct CalculatorStorage {
  storage: LinkedHashMap<String, GridCalculator>
}

impl CalculatorStorage {
  pub fn contains(&self, key: &str) -> bool {
    self.storage.contains_key(key)
  }

  pub fn get(&self, key: &str) -> Option<&GridCalculator> {
    self.storage.get(key)
  }

  pub fn iter(&self) -> impl Iterator<Item=(&String, &GridCalculator)> {
    self.storage.iter()
  }

  pub fn set<K: Into<String>>(&mut self, key: K, calculator: GridCalculator) {
    self.storage.insert(key.into(), calculator);
  }
}
