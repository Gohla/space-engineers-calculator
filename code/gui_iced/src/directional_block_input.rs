use std::collections::HashMap;

use iced::{Column, Element, Length, Row, Text};
use linked_hash_map::LinkedHashMap;

use secalc_core::data::blocks::{Block, BlockId, Blocks, GridSize};
use secalc_core::data::Data;
use secalc_core::grid::{Direction, GridCalculator};

use crate::data_bind::{DataBind, DataBindMessage};
use crate::widget::h2;

pub struct DirectionalBlockInput {
  small: LinkedHashMap<BlockId, LinkedHashMap<Direction, DataBind<u64>>>,
  large: LinkedHashMap<BlockId, LinkedHashMap<Direction, DataBind<u64>>>,
  labels: HashMap<BlockId, String>,
  label_width: Length,
  value_width: Length,
}

#[derive(Clone, Debug)]
pub struct DirectionalBlockInputMessage(GridSize, BlockId, Direction, DataBindMessage);

impl DirectionalBlockInput {
  pub fn new(label_width: Length, value_width: Length) -> Self {
    Self {
      small: LinkedHashMap::default(),
      large: LinkedHashMap::default(),
      labels: HashMap::default(),
      label_width,
      value_width,
    }
  }
  pub fn add_blocks<'a, T: 'a, I: Iterator<Item=&'a Block<T>>>(&mut self, data: &Data, blocks_iter: I) {
    let (small, large) = Blocks::small_and_large_sorted(blocks_iter);
    fn add_to_map<T>(value_width: Length, vec: Vec<&Block<T>>, map: &mut LinkedHashMap<BlockId, LinkedHashMap<Direction, DataBind<u64>>>) {
      for block in vec {
        let inner_map = map.entry(block.id.clone()).or_insert(LinkedHashMap::default());
        for direction in Direction::iter() {
          inner_map.insert(*direction, DataBind::new_without_label(0, "0", value_width, "#"));
        }
      }
    }
    for block in small.iter() {
      self.labels.insert(block.id.clone(), block.name(&data.localization).to_owned());
    }
    for block in large.iter() {
      self.labels.insert(block.id.clone(), block.name(&data.localization).to_owned());
    }
    add_to_map(self.value_width, small, &mut self.small);
    add_to_map(self.value_width, large, &mut self.large);
  }

  pub fn update(&mut self, message: DirectionalBlockInputMessage, calc: &mut GridCalculator) {
    let DirectionalBlockInputMessage(size, id, direction, m) = message;
    if let Some(inner_map) = self.map_for_size(size).get_mut(&id) {
      if let Some(data_bind) = inner_map.get_mut(&direction) {
        data_bind.update(m, calc.directional_blocks.entry(direction).or_default().entry(id.clone()).or_default())
      }
    }
  }

  pub fn view(&mut self) -> Element<DirectionalBlockInputMessage> {
    let input_small = Self::create_column(GridSize::Small, &mut self.small, &self.labels, self.label_width, self.value_width);
    let input_large = Self::create_column(GridSize::Large, &mut self.large, &self.labels, self.label_width, self.value_width);
    Row::new()
      .spacing(2)
      .padding(0)
      .push(input_small)
      .push(input_large)
      .into()
  }

  fn create_column<'a>(grid_size: GridSize, map: &'a mut LinkedHashMap<BlockId, LinkedHashMap<Direction, DataBind<u64>>>, labels: &HashMap<BlockId, String>, label_width: Length, _value_width: Length) -> Element<'a, DirectionalBlockInputMessage> {
    let mut column = Column::new()
      .push(h2(match grid_size { GridSize::Small => "Small grid", GridSize::Large => "Large grid" }));
    let mut first_row = Row::new()
      .width(Length::Shrink)
      .push(Text::new("").width(label_width))
      ;
    for direction in Direction::iter() {
      first_row = first_row.push(Text::new(format!("{:?}", direction)).width(Length::Units(50)))
    }
    column = column.push(first_row);
    for (id, inner_map) in map.iter_mut() {
      let mut row = Row::new();
      let label = labels.get(id).unwrap();
      row = row.push(Text::new(label).width(label_width));
      for (direction, data_bind) in inner_map {
        // Clone and copy before closure such that no reference is passed into the ('static) closure.
        let id = id.clone();
        let direction = *direction;
        row = row.push(data_bind.view().map(move |m| DirectionalBlockInputMessage(
          grid_size,
          // Clone again because this is a Fn closure that is callable multiple times: each call needs a separate clone and String does not implement Copy.
          id.clone(),
          direction,
          m
        )))
      }
      column = column.push(row)
    }
    column.into()
  }

  fn map_for_size(&mut self, size: GridSize) -> &mut LinkedHashMap<BlockId, LinkedHashMap<Direction, DataBind<u64>>> {
    match size { GridSize::Small => &mut self.small, GridSize::Large => &mut self.large }
  }
}
