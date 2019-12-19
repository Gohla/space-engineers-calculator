use std::ops::Deref;

use iced::{Element, Length};
use linked_hash_map::LinkedHashMap;

use secalc_core::data::blocks::{Block, BlockId, Blocks, GridSize};
use secalc_core::data::Data;
use secalc_core::grid::{Direction, GridCalculator};

use crate::data_bind::{DataBind, DataBindMessage};
use crate::view::{col, h3, lbl, row};

type InnerMap = LinkedHashMap<Direction, DataBind<u64>>;
type Map = LinkedHashMap<BlockId, (String, InnerMap)>;

pub struct DirectionalBlockInput {
  small: Map,
  large: Map,
  label_width: Length,
  input_width: Length,
  direction_label_width: Length,
}

#[derive(Clone, Debug)]
pub struct DirectionalBlockInputMessage(GridSize, BlockId, Direction, DataBindMessage);

impl DirectionalBlockInput {
  pub fn new(label_width: Length, input_width: Length, direction_label_width: Length) -> Self {
    Self {
      small: Map::default(),
      large: Map::default(),
      label_width,
      input_width,
      direction_label_width,
    }
  }

  pub fn add_blocks<'a, T: 'a, I: Iterator<Item=&'a Block<T>>>(&mut self, data: &Data, blocks_iter: I) {
    fn add_to_map<T>(data: &Data, input_width: Length, vec: Vec<&Block<T>>, map: &mut Map) {
      for block in vec {
        let label = block.name(&data.localization).to_owned();
        let (_, inner_map) = map.entry(block.id.clone()).or_insert((label, InnerMap::default()));
        for direction in Direction::iter() {
          inner_map.insert(*direction, DataBind::new(0, "0", input_width, "#"));
        }
      }
    }
    let (small, large) = Blocks::small_and_large_sorted(blocks_iter);
    add_to_map(data, self.input_width, small, &mut self.small);
    add_to_map(data, self.input_width, large, &mut self.large);
  }

  pub fn update(&mut self, message: DirectionalBlockInputMessage, calc: &mut GridCalculator) {
    let DirectionalBlockInputMessage(size, id, direction, m) = message;
    if let Some((_, inner_map)) = self.map_for_size(size).get_mut(&id) {
      if let Some(data_bind) = inner_map.get_mut(&direction) {
        data_bind.update(m, calc.directional_blocks.entry(direction).or_default().entry(id.clone()).or_default())
      }
    }
  }

  pub fn view(&mut self) -> Element<DirectionalBlockInputMessage> {
    let input_small = Self::create_column(&mut self.small, self.label_width, self.direction_label_width, GridSize::Small);
    let input_large = Self::create_column(&mut self.large, self.label_width, self.direction_label_width, GridSize::Large);
    row()
      .spacing(10)
      .padding(0)
      .push(input_small)
      .push(input_large)
      .into()
  }

  fn create_column(map: &mut Map, label_width: Length, direction_label_width: Length, grid_size: GridSize) -> Element<DirectionalBlockInputMessage> {
    let mut column = {
      let mut first_row = row()
        .width(Length::Shrink)
        .push(lbl("-").width(label_width))
        ;
      for direction in Direction::iter() {
        first_row = first_row.push(lbl(format!("{:?}", direction)).width(direction_label_width))
      }
      col().push(first_row)
    };

    for (id, (label, inner_map)) in map.iter_mut() {
      let mut row = row();
      row = row.push(lbl(label.deref()).width(label_width));
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
    col()
      .push(h3(match grid_size { GridSize::Small => "Small grid", GridSize::Large => "Large grid" }))
      .push(column)
      .into()
  }

  fn map_for_size(&mut self, size: GridSize) -> &mut Map {
    match size { GridSize::Small => &mut self.small, GridSize::Large => &mut self.large }
  }
}
