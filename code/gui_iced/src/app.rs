use std::fmt::Debug;

use iced::{Align, Application, Command, Element, HorizontalAlignment, Length, scrollable};

use secalc_core::data::Data;
use secalc_core::grid::{Direction, GridCalculated, GridCalculator};

use crate::block_input::{BlockInput, BlockInputMessage};
use crate::directional_block_input::{DirectionalBlockInput, DirectionalBlockInputMessage};
use crate::option_input::{OptionInput, OptionInputMessage};
use crate::view::{col, empty, h1, h2, h3, lbl, row, scl, val};

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
    let blocks_label_width = Length::Units(230);
    let blocks_input_width = Length::Units(35);
    let input_storage = {
      let mut blocks = BlockInput::new(blocks_label_width, blocks_input_width);
      blocks.add_blocks(&data, data.blocks.containers.values().filter(|c| c.details.store_any));
      blocks.add_blocks(&data, data.blocks.cockpits.values().filter(|c| c.details.has_inventory));
      blocks
    };
    let input_thrust = {
      let mut blocks = DirectionalBlockInput::new(blocks_label_width, blocks_input_width, Length::Units(49));
      blocks.add_blocks(&data, data.blocks.thrusters.values());
      blocks
    };
    let input_power = {
      let mut blocks = BlockInput::new(blocks_label_width, blocks_input_width);
      blocks.add_blocks(&data, data.blocks.hydrogen_engines.values());
      blocks.add_blocks(&data, data.blocks.reactors.values());
      blocks.add_blocks(&data, data.blocks.batteries.values());
      blocks
    };
    let input_hydrogen = {
      let mut blocks = BlockInput::new(blocks_label_width, blocks_input_width);
      blocks.add_blocks(&data, data.blocks.generators.values());
      blocks.add_blocks(&data, data.blocks.hydrogen_tanks.values());
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
    let input = Self::view_input(
      &mut self.input_options,
      &mut self.input_storage,
      &mut self.input_thrust,
      &mut self.input_power,
      &mut self.input_hydrogen,
      &mut self.input_scrollable
    );
    let result = Self::view_result(&self.result, &mut self.result_scrollable);
    let root: Element<_> = col()
      .padding(10)
      .push(row()
        .spacing(10)
        .align_items(Align::End)
        .push(h1("Space Engineers Calculator"))
        .push(h3(" https://github.com/Gohla/space_engineers_calc"))
      )
      .push(row()
        .spacing(20)
        .push(input)
        .push(result)
      ).into();
    root
  }
}

impl App {
  fn view_input<'a>(
    input_options: &'a mut OptionInput,
    input_storage: &'a mut BlockInput,
    input_thrust: &'a mut DirectionalBlockInput,
    input_power: &'a mut BlockInput,
    input_hydrogen: &'a mut BlockInput,
    scrollable_state: &'a mut scrollable::State
  ) -> Element<'a, Message> {
    scl(scrollable_state)
      .spacing(10)
      .padding(1)
      .push(col()
        .push(h2("Options"))
        .push(input_options.view().map(Message::InputOptionChange))
      )
      .push(col()
        .push(h2("Storage"))
        .push(input_storage.view().map(Message::InputStorageChange))
      )
      .push(col()
        .push(h2("Thrusters"))
        .push(input_thrust.view().map(Message::InputThrustChange))
      )
      .push(col()
        .push(h2("Power"))
        .push(input_power.view().map(Message::InputPowerChange))
      )
      .push(col()
        .push(h2("Hydrogen"))
        .push(input_hydrogen.view().map(Message::InputHydrogenChange))
      )
      .into()
  }


  fn view_result<'a>(result: &'a GridCalculated, scrollable_state: &'a mut scrollable::State) -> Element<'a, Message> {
    scl(scrollable_state)
      .spacing(10)
      .padding(1)
      .push(col()
        .push(h2("Mass"))
        .push(Self::view_result_mass(result))
      )
      .push(col()
        .push(h2("Volume"))
        .push(Self::view_result_volume(result))
      )
      .push(col()
        .push(h2("Items"))
        .push(Self::view_result_items(result))
      )
      .push(col()
        .push(h2("Acceleration (m/s^2)"))
        .push(Self::view_result_acceleration(result))
      )
      .push(col()
        .push(h2("Power"))
        .push(Self::view_result_power(result))
      )
      .push(col()
        .push(h2("Hydrogen"))
        .push(Self::view_result_hydrogen(result))
      )
      .into()
  }

  fn view_result_mass(result: &GridCalculated) -> Element<Message> {
    row()
      .spacing(10)
      .push(col()
        .spacing(1)
        .push(lbl("Empty"))
        .push(lbl("Filled"))
      )
      .push(col()
        .spacing(1)
        .push(val(format!("{:.0} kg", result.total_mass_empty)))
        .push(val(format!("{:.0} kg", result.total_mass_filled)))
      )
      .into()
  }

  fn view_result_volume(result: &GridCalculated) -> Element<Message> {
    row()
      .spacing(10)
      .push(col()
        .spacing(1)
        .push(lbl("Any"))
        .push(lbl("Ore"))
        .push(lbl("Ice"))
        .push(lbl("Ore-only"))
        .push(lbl("Ice-only"))
      )
      .push(col()
        .spacing(1)
        .push(val(format!("{:.0} L", result.total_volume_any)))
        .push(val(format!("{:.0} L", result.total_volume_ore)))
        .push(val(format!("{:.0} L", result.total_volume_ice)))
        .push(val(format!("{:.0} L", result.total_volume_ore_only)))
        .push(val(format!("{:.0} L", result.total_volume_ice_only)))
      )
      .into()
  }

  fn view_result_items(result: &GridCalculated) -> Element<Message> {
    row()
      .spacing(10)
      .push(col()
        .spacing(1)
        .push(lbl("Any"))
        .push(lbl("Ice"))
        .push(lbl("Steel Plates"))
      )
      .push(col()
        .spacing(1)
        .push(val(format!("{:.0} #", result.total_items_ore)))
        .push(val(format!("{:.0} #", result.total_items_ice)))
        .push(val(format!("{:.0} #", result.total_items_steel_plate)))
      )
      .into()
  }

  fn view_result_acceleration(result: &GridCalculated) -> Element<Message> {
    row()
      .spacing(10)
      .push({
        let mut column = col()
          .push(empty())
          .push(empty())
          ;
        for direction in Direction::iter() {
          column = column.push(val(format!("{:?}", direction)));
        }
        column
      })
      .push({
        let mut column = col()
          .push(lbl("Filled").horizontal_alignment(HorizontalAlignment::Center))
          .push(lbl("Gravity").horizontal_alignment(HorizontalAlignment::Center))
          ;
        for direction in Direction::iter() {
          if let Some(acceleration) = result.acceleration.get(direction) {
            column = column.push(val(format!("{:.2}", acceleration.acceleration_filled_gravity)))
          }
        }
        column
      })
      .push({
        let mut column = col()
          .push(empty())
          .push(lbl("No grav.").horizontal_alignment(HorizontalAlignment::Center))
          ;
        for direction in Direction::iter() {
          if let Some(acceleration) = result.acceleration.get(direction) {
            column = column.push(val(format!("{:.2}", acceleration.acceleration_filled_no_gravity)))
          }
        }
        column
      })
      .push({
        let mut column = col()
          .push(lbl("Empty").horizontal_alignment(HorizontalAlignment::Center))
          .push(lbl("Gravity").horizontal_alignment(HorizontalAlignment::Center))
          ;
        for direction in Direction::iter() {
          if let Some(acceleration) = result.acceleration.get(direction) {
            column = column.push(val(format!("{:.2}", acceleration.acceleration_empty_gravity)))
          }
        }
        column
      })
      .push({
        let mut column = col()
          .push(empty())
          .push(lbl("No grav.").horizontal_alignment(HorizontalAlignment::Center))
          ;
        for direction in Direction::iter() {
          if let Some(acceleration) = result.acceleration.get(direction) {
            column = column.push(val(format!("{:.2}", acceleration.acceleration_empty_no_gravity)))
          }
        }
        column
      })
      .into()
  }

  fn view_result_power(result: &GridCalculated) -> Element<Message> {
    row()
      .spacing(10)
      .push(col()
        .spacing(1)
        .push(lbl("Generation"))
        .push(lbl("Capacity: Batteries"))
        .push(empty())
        .push(lbl("Ide"))
        .push(lbl("Misc"))
        .push(lbl("+ Charge Jump Drives"))
        .push(lbl("+ Generators"))
        .push(lbl("+ Up/Down Thrusters"))
        .push(lbl("+ Front/Back Thrusters"))
        .push(lbl("+ Left/Right Thrusters"))
        .push(lbl("+ Charge Batteries"))
      )
      .push(col()
        .spacing(1)
        .push(val(format!("{:.2} MW", result.power_generation)))
        .push(val(format!("{:.2} MWh", result.power_capacity_battery)))
        .push(lbl("Consumption"))
        .push(val(format!("{:.2} MW", result.power_idle.consumption)))
        .push(val(format!("{:.2} MW", result.power_misc.consumption)))
        .push(val(format!("{:.2} MW", result.power_upto_jump_drive.consumption)))
        .push(val(format!("{:.2} MW", result.power_upto_generator.consumption)))
        .push(val(format!("{:.2} MW", result.power_upto_up_down_thruster.consumption)))
        .push(val(format!("{:.2} MW", result.power_upto_front_back_thruster.consumption)))
        .push(val(format!("{:.2} MW", result.power_upto_left_right_thruster.consumption)))
        .push(val(format!("{:.2} MW", result.power_upto_battery.consumption)))
      )
      .push(col()
        .spacing(1)
        .push(empty())
        .push(empty())
        .push(lbl("Balance"))
        .push(val(format!("{:.2} MW", result.power_idle.balance)))
        .push(val(format!("{:.2} MW", result.power_misc.balance)))
        .push(val(format!("{:.2} MW", result.power_upto_jump_drive.balance)))
        .push(val(format!("{:.2} MW", result.power_upto_generator.balance)))
        .push(val(format!("{:.2} MW", result.power_upto_up_down_thruster.balance)))
        .push(val(format!("{:.2} MW", result.power_upto_front_back_thruster.balance)))
        .push(val(format!("{:.2} MW", result.power_upto_left_right_thruster.balance)))
        .push(val(format!("{:.2} MW", result.power_upto_battery.balance)))
      )
      .push(col()
        .spacing(1)
        .push(empty())
        .push(empty())
        .push(lbl("Duration: Batteries"))
        .push(val(format!("{:.2} min", result.power_idle.duration)))
        .push(val(format!("{:.2} min", result.power_misc.duration)))
        .push(val(format!("{:.2} min", result.power_upto_jump_drive.duration)))
        .push(val(format!("{:.2} min", result.power_upto_generator.duration)))
        .push(val(format!("{:.2} min", result.power_upto_up_down_thruster.duration)))
        .push(val(format!("{:.2} min", result.power_upto_front_back_thruster.duration)))
        .push(val(format!("{:.2} min", result.power_upto_left_right_thruster.duration)))
        .push(val(format!("{:.2} min", result.power_upto_battery.duration)))
      )
      .into()
  }

  fn view_result_hydrogen(result: &GridCalculated) -> Element<Message> {
    row()
      .spacing(10)
      .push(col()
        .spacing(1)
        .push(lbl("Generation"))
        .push(lbl("Capacity: Engines"))
        .push(lbl("Capacity: Tanks"))
        .push(empty())
        .push(lbl("Idle"))
        .push(lbl("Engines"))
        .push(lbl("+ Up/Down Thrusters"))
        .push(lbl("+ Front/Back Thrusters"))
        .push(lbl("+ Left/Right Thrusters"))
      )
      .push(col()
        .spacing(1)
        .push(val(format!("{:.0} L/s", result.hydrogen_generation)))
        .push(val(format!("{:.0} L", result.hydrogen_capacity_engine)))
        .push(val(format!("{:.0} L", result.hydrogen_capacity_tank)))
        .push(lbl("Consumption"))
        .push(val(format!("{:.1} L/s", result.hydrogen_idle.consumption)))
        .push(val(format!("{:.1} L/s", result.hydrogen_engine.consumption)))
        .push(val(format!("{:.1} L/s", result.hydrogen_upto_up_down_thruster.consumption)))
        .push(val(format!("{:.1} L/s", result.hydrogen_upto_front_back_thruster.consumption)))
        .push(val(format!("{:.1} L/s", result.hydrogen_upto_left_right_thruster.consumption)))
      )
      .push(col()
        .spacing(1)
        .push(empty())
        .push(empty())
        .push(empty())
        .push(lbl("Balance"))
        .push(val(format!("{:.1} L/s", result.hydrogen_idle.balance)))
        .push(val(format!("{:.1} L/s", result.hydrogen_engine.balance)))
        .push(val(format!("{:.1} L/s", result.hydrogen_upto_up_down_thruster.balance)))
        .push(val(format!("{:.1} L/s", result.hydrogen_upto_front_back_thruster.balance)))
        .push(val(format!("{:.1} L/s", result.hydrogen_upto_left_right_thruster.balance)))
      )
      .push(col()
        .spacing(1)
        .push(empty())
        .push(empty())
        .push(empty())
        .push(lbl("Duration: Tanks"))
        .push(val(format!("{:.2} min", result.hydrogen_idle.duration)))
        .push(val(format!("{:.2} min", result.hydrogen_engine.duration)))
        .push(val(format!("{:.2} min", result.hydrogen_upto_up_down_thruster.duration)))
        .push(val(format!("{:.2} min", result.hydrogen_upto_front_back_thruster.duration)))
        .push(val(format!("{:.2} min", result.hydrogen_upto_left_right_thruster.duration)))
      )
      .into()
  }
}