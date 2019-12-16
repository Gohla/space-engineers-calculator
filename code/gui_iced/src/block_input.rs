use iced::{Column, Element, Length, Row};
use linked_hash_map::LinkedHashMap;

use secalc_core::data::blocks::{Block, BlockId, Blocks, GridSize};
use secalc_core::data::Data;
use secalc_core::grid::GridCalculator;

use crate::data_bind::{DataBind, DataBindMessage};
use crate::widget::h2;

#[derive(Default)]
pub struct BlockInput {
  small: LinkedHashMap<BlockId, DataBind<u64>>,
  large: LinkedHashMap<BlockId, DataBind<u64>>,
}

#[derive(Clone, Debug)]
pub struct BlockInputMessage(BlockId, GridSize, DataBindMessage);

impl BlockInput {
  pub fn add_blocks<'a, T: 'a, I: Iterator<Item=&'a Block<T>>>(&mut self, data: &Data, label_width: Length, value_width: Length, blocks_iter: I) {
    let (small, large) = Blocks::small_and_large_sorted(blocks_iter);
    fn add_to_map<T>(data: &Data, label_width: Length, value_width: Length, vec: Vec<&Block<T>>, map: &mut LinkedHashMap<BlockId, DataBind<u64>>) {
      map.extend(vec.into_iter()
        .map(|b| (b.id.clone(), DataBind::new(b.name(&data.localization), label_width, 0, "0", value_width, "#")))
      );
    }
    add_to_map(data, label_width, value_width, small, &mut self.small);
    add_to_map(data, label_width, value_width, large, &mut self.large);
  }

  pub fn update(&mut self, message: BlockInputMessage, calc: &mut GridCalculator) {
    let BlockInputMessage(id, size, m) = message;
    if let Some(data_bind) = self.map_for_size(size).get_mut(&id) {
      data_bind.update(m, calc.blocks.entry(id.clone()).or_default())
    }
  }

  pub fn view(&mut self) -> Element<BlockInputMessage> {
    fn create_column(grid_size: GridSize, map: &mut LinkedHashMap<BlockId, DataBind<u64>>) -> Element<BlockInputMessage> {
      let mut column = {
        let label = match grid_size { GridSize::Small => "Small grid", GridSize::Large => "Large grid" };
        Column::new().push(h2(label))
      };
      for (id, data_bind) in map {
        let id = id.clone(); // Clone to simplify lifetimes: we have an owned String now.
        column = column.push(data_bind.view().map(move |m| BlockInputMessage(
          // Clone again because this is a Fn closure that is callable multiple times: each call needs a separate clone.
          id.clone(),
          grid_size,
          m
        )))
      }
      column.into()
    }
    let input_small = create_column(GridSize::Small, &mut self.small);
    let input_large = create_column(GridSize::Large, &mut self.large);
    Row::new()
      .spacing(2)
      .padding(0)
      .push(input_small)
      .push(input_large)
      .into()
  }

  fn map_for_size(&mut self, size: GridSize) -> &mut LinkedHashMap<BlockId, DataBind<u64>> {
    match size { GridSize::Small => &mut self.small, GridSize::Large => &mut self.large }
  }
}
