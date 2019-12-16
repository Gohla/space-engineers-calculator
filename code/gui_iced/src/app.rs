use std::fmt::Debug;

use iced::{Application, Color, Column, Command, Element, HorizontalAlignment, Length, Row, scrollable, Scrollable, Text};
use linked_hash_map::LinkedHashMap;

use secalc_core::data::blocks::{Block, BlockId, Blocks, GridSize};
use secalc_core::data::Data;
use secalc_core::grid::{GridCalculated, GridCalculator};

use crate::data_bind::{DataBind, DataBindMessage};

// Input options

macro_rules! create_input_options {
  ($label_length:expr; $input_length:expr; $($field:ident, $type:ty, $message:ident, $label:expr, $format:expr, $unit:expr);*) => {
    struct InputOptions {
      $($field: DataBind<$type>,)*
    }

    impl InputOptions {
      fn new(calc: &GridCalculator) -> Self {
        Self {
          $($field: DataBind::new($label, $label_length, calc.$field, format!($format, calc.$field), $input_length, $unit),)*
        }
      }
    }

    #[derive(Clone, Debug)]
    pub enum InputOptionMessage {
      $($message(DataBindMessage),)*
    }

    impl InputOptions {
      fn update(&mut self, message: InputOptionMessage, calc: &mut GridCalculator) {
        match message {
          $(InputOptionMessage::$message(m) => self.$field.update(m, &mut calc.$field),)*
        }
      }

      pub fn view(&mut self) -> impl IntoIterator<Item=Element<InputOptionMessage>> {
        vec![
          $(self.$field.view().map(move |s| InputOptionMessage::$message(s)),)*
        ]
      }
    }
  }
}

create_input_options!(Length::Units(250); Length::Units(100);
  gravity_multiplier, f64, GravityMultiplier, "Gravity Multiplier", "{:.1}", "*";
  container_multiplier, f64, ContainerMultiplier, "Container Multiplier", "{:.1}", "*";
  planetary_influence, f64, PlanetaryInfluence, "Planetary Influence", "{:.1}", "*";
  additional_mass, f64, AdditionalMass, "Additional Mass", "{}", "kg";
  ice_only_fill, f64, IceOnlyFill, "Ice-only-fill", "{:.1}", "%";
  ore_only_fill, f64, OreOnlyFill, "Ore-only-fill", "{:.1}", "%";
  any_fill_with_ice, f64, AnyFillWithIce, "Any-fill with Ice", "{:.1}", "%";
  any_fill_with_ore, f64, AnyFillWithOre, "Any-fill with Ore", "{:.1}", "%";
  any_fill_with_steel_plates, f64, AnyFillWithSteelPlates, "Any-fill with Steel Plates", "{:.1}", "%"
);


// Input blocks

#[derive(Default)]
pub struct InputBlocks {
  small: LinkedHashMap<BlockId, DataBind<u64>>,
  large: LinkedHashMap<BlockId, DataBind<u64>>,
}

#[derive(Clone, Debug)]
pub struct InputBlocksMessage(BlockId, GridSize, DataBindMessage);

impl InputBlocks {
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

  pub fn update(&mut self, message: InputBlocksMessage, calc: &mut GridCalculator) {
    let InputBlocksMessage(id, size, m) = message;
    if let Some(data_bind) = self.map_for_size(size).get_mut(&id) {
      data_bind.update(m, calc.blocks.entry(id.clone()).or_default())
    }
  }

  pub fn view(&mut self) -> Element<InputBlocksMessage> {
    fn create_column(grid_size: GridSize, map: &mut LinkedHashMap<BlockId, DataBind<u64>>) -> Element<InputBlocksMessage> {
      let mut column = {
        let label = match grid_size { GridSize::Small => "Small grid", GridSize::Large => "Large grid" };
        Column::new().push(h2(label))
      };
      for (id, data_bind) in map {
        let id = id.clone(); // Clone to simplify lifetimes: we have an owned String now.
        column = column.push(data_bind.view().map(move |m| InputBlocksMessage(
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


// Application

pub struct App {
  input_options: InputOptions,
  input_storage: InputBlocks,
  input_power: InputBlocks,
  input_hydrogen: InputBlocks,

  data: Data,
  calculator: GridCalculator,
  result: GridCalculated,

  input_scrollable: scrollable::State,
  result_scrollable: scrollable::State,
}

impl Default for App {
  fn default() -> Self {
    let data_bytes: &[u8] = include_bytes!("../../../data/data.json");
    let data = Data::from_json(data_bytes).expect("Cannot read data");
    let calculator = GridCalculator::default();
    let result = calculator.calculate(&data);

    let input_options = InputOptions::new(&calculator);
    let blocks_label_width = Length::Units(250);
    let blocks_value_width = Length::Units(35);
    let input_storage = {
      let mut blocks = InputBlocks::default();
      blocks.add_blocks(&data, blocks_label_width, blocks_value_width, data.blocks.containers.values().filter(|c| c.details.store_any));
      blocks.add_blocks(&data, blocks_label_width, blocks_value_width, data.blocks.cockpits.values().filter(|c| c.details.has_inventory));
      blocks
    };
    let input_power = {
      let mut blocks = InputBlocks::default();
      blocks.add_blocks(&data, blocks_label_width, blocks_value_width, data.blocks.hydrogen_engines.values());
      blocks.add_blocks(&data, blocks_label_width, blocks_value_width, data.blocks.reactors.values());
      blocks.add_blocks(&data, blocks_label_width, blocks_value_width, data.blocks.batteries.values());
      blocks
    };
    let input_hydrogen = {
      let mut blocks = InputBlocks::default();
      blocks.add_blocks(&data, blocks_label_width, blocks_value_width, data.blocks.generators.values());
      blocks.add_blocks(&data, blocks_label_width, blocks_value_width, data.blocks.hydrogen_tanks.values());
      blocks
    };

    Self {
      input_options,
      input_storage,
      input_power,
      input_hydrogen,
      data,
      calculator,
      result,
      input_scrollable: scrollable::State::default(),
      result_scrollable: scrollable::State::default(),
    }
  }
}

#[derive(Clone, Debug)]
pub enum Message {
  InputOptionChange(InputOptionMessage),
  InputStorageChange(InputBlocksMessage),
  InputPowerChange(InputBlocksMessage),
  InputHydrogenChange(InputBlocksMessage),
}

impl Application for App {
  type Message = Message;

  fn new() -> (Self, Command<Message>) {
    (Self::default(), Command::none())
  }

  fn title(&self) -> String {
    String::from("Space Engineers Calculator")
  }


  fn update(&mut self, message: Message) -> Command<Message> {
    match message {
      Message::InputOptionChange(m) => self.input_options.update(m, &mut self.calculator),
      Message::InputStorageChange(m) => self.input_storage.update(m, &mut self.calculator),
      Message::InputPowerChange(m) => self.input_power.update(m, &mut self.calculator),
      Message::InputHydrogenChange(m) => self.input_hydrogen.update(m, &mut self.calculator),
    }
    self.result = self.calculator.calculate(&self.data);
    Command::none()
  }

  fn view(&mut self) -> Element<Message> {
    // Input
    let input = Scrollable::new(&mut self.input_scrollable)
      .spacing(1)
      .padding(0);
    // - Options
    let mut input = input.push(h1("Options"));
    for elem in self.input_options.view() {
      input = input.push(elem.map(Message::InputOptionChange));
    }
    // - Storage
    let input = input
      .push(h1("Storage"))
      .push(self.input_storage.view().map(Message::InputStorageChange))
      ;
    // - TODO: Thrusters
    let input =
      input.push(h1("Thrusters"))
      ;
    // - Power
    let input = input
      .push(h1("Power"))
      .push(self.input_power.view().map(Message::InputPowerChange))
      ;
    // - Hydrogen
    let input = input
      .push(h1("Hydrogen"))
      .push(self.input_hydrogen.view().map(Message::InputHydrogenChange))
      ;

    // Result
    let label_width = Length::Units(110);
    let value_width = Length::Units(100);
    let result = Scrollable::new(&mut self.result_scrollable)
      .spacing(1)
      .padding(0)
      .push(h1("Mass"))
      .push_labelled("Empty", label_width, format!("{:.0}", self.result.total_mass_empty), value_width, "kg")
      .push_labelled("Filled", label_width, format!("{:.0}", self.result.total_mass_filled), value_width, "kg")
      .push(h1("Volume"))
      .push_labelled("Any", label_width, format!("{:.0}", self.result.total_volume_any), value_width, "L")
      .push_labelled("Ore", label_width, format!("{:.0}", self.result.total_volume_ore), value_width, "L")
      .push_labelled("Ice", label_width, format!("{:.0}", self.result.total_volume_ice), value_width, "L")
      .push_labelled("Ore-only", label_width, format!("{:.0}", self.result.total_volume_ore_only), value_width, "L")
      .push_labelled("Ice-only", label_width, format!("{:.0}", self.result.total_volume_ice_only), value_width, "L")
      .push(h1("Items"))
      .push_labelled("Ore", label_width, format!("{:.0}", self.result.total_items_ore), value_width, "#")
      .push_labelled("Ice", label_width, format!("{:.0}", self.result.total_items_ice), value_width, "#")
      .push_labelled("Steel Plates", label_width, format!("{:.0}", self.result.total_items_steel_plate), value_width, "#")
      .push(h1("Force & Acceleration"))
      .push(h1("Power"))
      .push(h1("Hydrogen"))
      ;

    // Root
    let root: Element<_> = Row::new()
      .spacing(10)
      .padding(5)
      .push(input)
      .push(result)
      .into();
    root.explain(Color::BLACK)
  }
}


// Helper functions

const H1_SIZE: u16 = 28;
const H2_SIZE: u16 = 24;

fn h1<L: Into<String>>(label: L) -> Text {
  Text::new(label).size(H1_SIZE)
}

fn h2<L: Into<String>>(label: L) -> Text {
  Text::new(label).size(H2_SIZE)
}


// Helper extension traits

trait ScrollableExt {
  fn push_labelled<L: Into<String>, V: Into<String>, U: Into<String>>(self, label: L, label_width: Length, value: V, value_width: Length, unit: U) -> Self;
}

impl<'a, Message: 'static + Clone> ScrollableExt for Scrollable<'a, Message> {
  fn push_labelled<L: Into<String>, V: Into<String>, U: Into<String>>(self, label: L, label_width: Length, value: V, value_width: Length, unit: U) -> Self {
    let text = Text::new(label)
      .width(label_width)
      ;
    let value = Text::new(value)
      .width(value_width)
      .horizontal_alignment(HorizontalAlignment::Right);
    let unit = Text::new(unit)
      .width(Length::Shrink)
      ;

    self.push(Row::new()
      .push(text)
      .push(value)
      .push(unit)
      .padding(1)
      .spacing(2)
    )
  }
}
