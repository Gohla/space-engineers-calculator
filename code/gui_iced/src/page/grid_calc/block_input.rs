use std::ops::Deref;

use iced::{Align, Element, Length};
use linked_hash_map::LinkedHashMap;

use secalc_core::data::blocks::{Block, BlockId, Blocks, GridSize};
use secalc_core::data::Data;
use secalc_core::grid::GridCalculator;

use crate::data_bind::{DataBind, DataBindMessage};
use crate::view::{col, h3, lbl, row};

type Map = LinkedHashMap<BlockId, (String, DataBind<u64>)>;

pub struct BlockInput {
  small: Map,
  large: Map,
  label_width: Length,
  input_width: Length,
}

#[derive(Clone, Debug)]
pub struct BlockInputMessage(BlockId, GridSize, DataBindMessage);

impl BlockInput {
  pub fn new(label_width: Length, input_width: Length) -> Self {
    Self {
      small: Map::default(),
      large: Map::default(),
      label_width,
      input_width,
    }
  }

  pub fn add_blocks<'a, T: 'a, I: Iterator<Item=&'a Block<T>>>(&mut self, data: &Data, blocks_iter: I) {
    fn add_to_map<T>(data: &Data, input_width: Length, vec: Vec<&Block<T>>, map: &mut Map) {
      map.extend(vec.into_iter()
        .map(|b| (b.id.clone(), (b.name(&data.localization).to_owned(), DataBind::new(0, "0", input_width, "#"))))
      );
    }
    let (small, large) = Blocks::small_and_large_sorted(blocks_iter);
    add_to_map(data, self.input_width, small, &mut self.small);
    add_to_map(data, self.input_width, large, &mut self.large);
  }

  pub fn update(&mut self, message: BlockInputMessage, calc: &mut GridCalculator) {
    let BlockInputMessage(id, size, m) = message;
    if let Some((_, data_bind)) = self.map_for_size(size).get_mut(&id) {
      data_bind.update(m, calc.blocks.entry(id.clone()).or_default())
    }
  }

  pub fn reload(&mut self, calc: &GridCalculator) {
    for (id, (_, data_bind)) in self.small.iter_mut().chain(self.large.iter_mut()) {
      let count = calc.blocks.get(id).map_or(0, |c| *c);
      data_bind.reload(format!("{}", count));
    }
  }

  pub fn view(&mut self) -> Element<BlockInputMessage> {
    fn create_column(map: &mut Map, label_width: Length, grid_size: GridSize) -> Element<BlockInputMessage> {
      let mut column = col();
      for (id, (label, data_bind)) in map {
        let id = id.clone(); // Clone before closure so that we are not passing references into 'static closure.
        column = column.push(row().align_items(Align::Center)
          .push(lbl(label.deref()).width(label_width))
          .push(data_bind.view().map(move |m| BlockInputMessage(
            // Clone again because this is a Fn closure that is callable multiple times: each call needs a separate clone and String does not implement Copy.
            id.clone(),
            grid_size,
            m
          )))
        )
      }
      col()
        .push(h3(match grid_size { GridSize::Small => "Small grid", GridSize::Large => "Large grid" }))
        .push(column)
        .into()
    }
    let input_small = create_column(&mut self.small, self.label_width, GridSize::Small);
    let input_large = create_column(&mut self.large, self.label_width, GridSize::Large);
    row()
      .spacing(10)
      .padding(0)
      .push(input_small)
      .push(input_large)
      .into()
  }

  fn map_for_size(&mut self, size: GridSize) -> &mut Map {
    match size { GridSize::Small => &mut self.small, GridSize::Large => &mut self.large }
  }
}
