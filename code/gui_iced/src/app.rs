use std::fmt::Debug;

use iced::{Application, Color, Command, Element, Length, Row, scrollable, Scrollable, Text, Column};

use secalc_core::data::Data;
use secalc_core::grid::{GridCalculated, GridCalculator, Direction};

use crate::block_input::{BlockInput, BlockInputMessage};
use crate::directional_block_input::{DirectionalBlockInput, DirectionalBlockInputMessage};
use crate::option_input::{OptionInput, OptionInputMessage};
use crate::widget::{h1, ScrollableExt};

pub struct App {
  input_options: OptionInput,
  input_storage: BlockInput,
  input_thrust: DirectionalBlockInput,
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
    let input_thrust = {
      let mut blocks = DirectionalBlockInput::new(blocks_label_width, blocks_value_width);
      blocks.add_blocks(&data, data.blocks.thrusters.values());
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
      input_thrust,
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
  InputThrustChange(DirectionalBlockInputMessage),
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
      Message::InputThrustChange(m) => self.input_thrust.update(m, &mut self.calculator),
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
      .padding(1)
      .width(Length::Shrink)
      ;
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
    let input = input
      .push(h1("Thrusters"))
      .push(self.input_thrust.view().map(Message::InputThrustChange))
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
    let result_acceleration: Element<_> = {
      let col_width = Length::Units(70);
      let mut row = Row::new()
        .spacing(1)
        .width(Length::Shrink)
        .push(Text::new("Acceleration (m/s^2)").width(Length::Shrink))
        .push(
          Column::new().width(Length::Units(100))
            .push(Text::new("-"))
            .push(Text::new("-"))
            .push(Text::new("No gravity"))
            .push(Text::new("Gravity"))
        );
      for direction in Direction::iter() {
        if let Some(accel) = self.result.acceleration.get(direction) {
          row = row
            .push(
              Column::new().width(col_width)
                .push(Text::new(format!("{:?}", direction)))
                .push(Text::new("Empty"))
                .push(Text::new(format!("{:.2}", accel.acceleration_empty_no_gravity)))
                .push(Text::new(format!("{:.2}", accel.acceleration_empty_gravity)))
            )
            .push(
              Column::new().width(col_width)
                .push(Text::new("-"))
                .push(Text::new("Filled"))
                .push(Text::new(format!("{:.2}", accel.acceleration_filled_no_gravity)))
                .push(Text::new(format!("{:.2}", accel.acceleration_filled_gravity)))
            )
        }
      }
      row.into()
    };

    let result_power: Element<_> = {
      fn t<L: Into<String>>(label: L) -> Text { Text::new(label).width(Length::Shrink) }
      let row = Row::new().width(Length::Shrink).spacing(10)
        .push(Column::new().width(Length::Shrink)
          .push(t("Generation (MW)"))
          .push(t("Capacity: Batteries (MWh)"))
          .push(t("-"))
          .push(t("Ide (MW)"))
          .push(t("Misc (MW)"))
          .push(t("+Charge Jump Drives (MW)"))
          .push(t("+Generators (MW)"))
          .push(t("+Up/Down Thrusters (MW)"))
          .push(t("+Front/Back Thrusters (MW)"))
          .push(t("+Left/Right Thrusters (MW)"))
          .push(t("+Charge Batteries (MW)"))
        )
        .push(Column::new().width(Length::Shrink)
          .push(t(format!("{:.2}", self.result.power_generation)))
          .push(t(format!("{:.2}", self.result.power_capacity_battery)))
          .push(t("Consumption"))
          .push(t(format!("{:.2}", self.result.power_idle.consumption)))
          .push(t(format!("{:.2}", self.result.power_misc.consumption)))
          .push(t(format!("{:.2}", self.result.power_upto_jump_drive.consumption)))
          .push(t(format!("{:.2}", self.result.power_upto_generator.consumption)))
          .push(t(format!("{:.2}", self.result.power_upto_up_down_thruster.consumption)))
          .push(t(format!("{:.2}", self.result.power_upto_front_back_thruster.consumption)))
          .push(t(format!("{:.2}", self.result.power_upto_left_right_thruster.consumption)))
          .push(t(format!("{:.2}", self.result.power_upto_battery.consumption)))
        )
        .push(Column::new().width(Length::Shrink)
          .push(t("-"))
          .push(t("-"))
          .push(t("Balance"))
          .push(t(format!("{:.2}", self.result.power_idle.balance)))
          .push(t(format!("{:.2}", self.result.power_misc.balance)))
          .push(t(format!("{:.2}", self.result.power_upto_jump_drive.balance)))
          .push(t(format!("{:.2}", self.result.power_upto_generator.balance)))
          .push(t(format!("{:.2}", self.result.power_upto_up_down_thruster.balance)))
          .push(t(format!("{:.2}", self.result.power_upto_front_back_thruster.balance)))
          .push(t(format!("{:.2}", self.result.power_upto_left_right_thruster.balance)))
          .push(t(format!("{:.2}", self.result.power_upto_battery.balance)))
        )
        .push(Column::new().width(Length::Shrink)
          .push(t("-"))
          .push(t("-"))
          .push(t("Duration: Batteries (min)"))
          .push(t(format!("{:.2}", self.result.power_idle.duration)))
          .push(t(format!("{:.2}", self.result.power_misc.duration)))
          .push(t(format!("{:.2}", self.result.power_upto_jump_drive.duration)))
          .push(t(format!("{:.2}", self.result.power_upto_generator.duration)))
          .push(t(format!("{:.2}", self.result.power_upto_up_down_thruster.duration)))
          .push(t(format!("{:.2}", self.result.power_upto_front_back_thruster.duration)))
          .push(t(format!("{:.2}", self.result.power_upto_left_right_thruster.duration)))
          .push(t(format!("{:.2}", self.result.power_upto_battery.duration)))
        )
        ;
      row.into()
    };

    let result_hydrogen: Element<_> = {
      fn t<L: Into<String>>(label: L) -> Text { Text::new(label).width(Length::Shrink) }
      let row = Row::new().width(Length::Shrink).spacing(10)
        .push(Column::new().width(Length::Shrink)
          .push(t("Generation (L/s)"))
          .push(t("Capacity: Engines (L)"))
          .push(t("Capacity: Tanks (L)"))
          .push(t("-"))
          .push(t("Ide (L/s)"))
          .push(t("Engines (L/s)"))
          .push(t("+Up/Down Thrusters (L/s)"))
          .push(t("+Front/Back Thrusters (L/s)"))
          .push(t("+Left/Right Thrusters (L/s)"))
        )
        .push(Column::new().width(Length::Shrink)
          .push(t(format!("{:.2}", self.result.hydrogen_generation)))
          .push(t(format!("{:.2}", self.result.hydrogen_capacity_engine)))
          .push(t(format!("{:.2}", self.result.hydrogen_capacity_tank)))
          .push(t("Consumption"))
          .push(t(format!("{:.2}", self.result.hydrogen_idle.consumption)))
          .push(t(format!("{:.2}", self.result.hydrogen_engine.consumption)))
          .push(t(format!("{:.2}", self.result.hydrogen_upto_up_down_thruster.consumption)))
          .push(t(format!("{:.2}", self.result.hydrogen_upto_front_back_thruster.consumption)))
          .push(t(format!("{:.2}", self.result.hydrogen_upto_left_right_thruster.consumption)))
        )
        .push(Column::new().width(Length::Shrink)
          .push(t("-"))
          .push(t("-"))
          .push(t("-"))
          .push(t("Balance"))
          .push(t(format!("{:.2}", self.result.hydrogen_idle.balance)))
          .push(t(format!("{:.2}", self.result.hydrogen_engine.balance)))
          .push(t(format!("{:.2}", self.result.hydrogen_upto_up_down_thruster.balance)))
          .push(t(format!("{:.2}", self.result.hydrogen_upto_front_back_thruster.balance)))
          .push(t(format!("{:.2}", self.result.hydrogen_upto_left_right_thruster.balance)))
        )
        .push(Column::new().width(Length::Shrink)
          .push(t("-"))
          .push(t("-"))
          .push(t("-"))
          .push(t("Duration: Tanks (min)"))
          .push(t(format!("{:.2}", self.result.hydrogen_idle.duration)))
          .push(t(format!("{:.2}", self.result.hydrogen_engine.duration)))
          .push(t(format!("{:.2}", self.result.hydrogen_upto_up_down_thruster.duration)))
          .push(t(format!("{:.2}", self.result.hydrogen_upto_front_back_thruster.duration)))
          .push(t(format!("{:.2}", self.result.hydrogen_upto_left_right_thruster.duration)))
        )
        ;
      row.into()
    };

    let label_width = Length::Units(110);
    let value_width = Length::Units(100);
    let result = Scrollable::new(&mut self.result_scrollable)
      .spacing(1)
      .padding(1)
      .width(Length::Shrink)
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
      .push(result_acceleration)
      .push(h1("Power"))
      .push(result_power)
      .push(h1("Hydrogen"))
      .push(result_hydrogen)
      ;

    // Root
    let root: Element<_> = Row::new()
      .spacing(25)
      .padding(10)
      .push(input)
      .push(result)
      .into();
    root.explain(Color::BLACK)
  }
}
