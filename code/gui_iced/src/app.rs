use std::fmt::Debug;

use iced::{Application, Color, Command, Element, Length, Row, scrollable, Scrollable};

use secalc_core::data::Data;
use secalc_core::grid::{GridCalculated, GridCalculator};

use crate::block_input::{BlockInput, BlockInputMessage};
use crate::option_input::{OptionInput, OptionInputMessage};
use crate::widget::{h1, ScrollableExt};

pub struct App {
  input_options: OptionInput,
  input_storage: BlockInput,
  input_power: BlockInput,
  input_hydrogen: BlockInput,

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

    let input_options = OptionInput::new(&calculator);
    let blocks_label_width = Length::Units(250);
    let blocks_value_width = Length::Units(35);
    let input_storage = {
      let mut blocks = BlockInput::default();
      blocks.add_blocks(&data, blocks_label_width, blocks_value_width, data.blocks.containers.values().filter(|c| c.details.store_any));
      blocks.add_blocks(&data, blocks_label_width, blocks_value_width, data.blocks.cockpits.values().filter(|c| c.details.has_inventory));
      blocks
    };
    let input_power = {
      let mut blocks = BlockInput::default();
      blocks.add_blocks(&data, blocks_label_width, blocks_value_width, data.blocks.hydrogen_engines.values());
      blocks.add_blocks(&data, blocks_label_width, blocks_value_width, data.blocks.reactors.values());
      blocks.add_blocks(&data, blocks_label_width, blocks_value_width, data.blocks.batteries.values());
      blocks
    };
    let input_hydrogen = {
      let mut blocks = BlockInput::default();
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
  InputOptionChange(OptionInputMessage),
  InputStorageChange(BlockInputMessage),
  InputPowerChange(BlockInputMessage),
  InputHydrogenChange(BlockInputMessage),
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
