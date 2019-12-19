use std::fmt::Debug;

use iced::{Application, Command, Element, Length, scrollable, Align};

use secalc_core::data::Data;
use secalc_core::grid::{Direction, GridCalculated, GridCalculator};

use crate::block_input::{BlockInput, BlockInputMessage};
use crate::directional_block_input::{DirectionalBlockInput, DirectionalBlockInputMessage};
use crate::option_input::{OptionInput, OptionInputMessage};
use crate::view::{col, ColumnExt, h1, h2, lbl, row, scl, h3};

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
    let blocks_label_width = Length::Units(210);
    let blocks_input_width = Length::Units(35);
    let input_storage = {
      let mut blocks = BlockInput::new(blocks_label_width, blocks_input_width);
      blocks.add_blocks(&data, data.blocks.containers.values().filter(|c| c.details.store_any));
      blocks.add_blocks(&data, data.blocks.cockpits.values().filter(|c| c.details.has_inventory));
      blocks
    };
    let input_thrust = {
      let mut blocks = DirectionalBlockInput::new(blocks_label_width, blocks_input_width, Length::Units(50));
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
    // Input
    let input = scl(&mut self.input_scrollable)
      .spacing(10)
      .padding(1)
      .push(col()
        .push(h2("Options"))
        .push(self.input_options.view().map(Message::InputOptionChange))
      )
      .push(col()
        .push(h2("Storage"))
        .push(self.input_storage.view().map(Message::InputStorageChange))
      )
      .push(col()
        .push(h2("Thrusters"))
        .push(self.input_thrust.view().map(Message::InputThrustChange))
      )
      .push(col()
        .push(h2("Power"))
        .push(self.input_power.view().map(Message::InputPowerChange))
      )
      .push(col()
        .push(h2("Hydrogen"))
        .push(self.input_hydrogen.view().map(Message::InputHydrogenChange))
      )
      ;

    // Result
    let result_acceleration: Element<_> = {
      let col_width = Length::Units(70);
      let mut row = row()
        .spacing(1)
        .width(Length::Shrink)
        .push(lbl("Acceleration (m/s^2)").width(Length::Shrink))
        .push(
          col().width(Length::Units(100))
            .push(lbl("-"))
            .push(lbl("-"))
            .push(lbl("No gravity"))
            .push(lbl("Gravity"))
        );
      for direction in Direction::iter() {
        if let Some(accel) = self.result.acceleration.get(direction) {
          row = row
            .push(
              col().width(col_width)
                .push(lbl(format!("{:?}", direction)))
                .push(lbl("Empty"))
                .push(lbl(format!("{:.2}", accel.acceleration_empty_no_gravity)))
                .push(lbl(format!("{:.2}", accel.acceleration_empty_gravity)))
            )
            .push(
              col().width(col_width)
                .push(lbl("-"))
                .push(lbl("Filled"))
                .push(lbl(format!("{:.2}", accel.acceleration_filled_no_gravity)))
                .push(lbl(format!("{:.2}", accel.acceleration_filled_gravity)))
            )
        }
      }
      row.into()
    };

    let result_power: Element<_> = {
      let row = row().spacing(10)
        .push(col()
          .push(lbl("Generation (MW)"))
          .push(lbl("Capacity: Batteries (MWh)"))
          .push(lbl("-"))
          .push(lbl("Ide (MW)"))
          .push(lbl("Misc (MW)"))
          .push(lbl("+Charge Jump Drives (MW)"))
          .push(lbl("+Generators (MW)"))
          .push(lbl("+Up/Down Thrusters (MW)"))
          .push(lbl("+Front/Back Thrusters (MW)"))
          .push(lbl("+Left/Right Thrusters (MW)"))
          .push(lbl("+Charge Batteries (MW)"))
        )
        .push(col()
          .push(lbl(format!("{:.2}", self.result.power_generation)))
          .push(lbl(format!("{:.2}", self.result.power_capacity_battery)))
          .push(lbl("Consumption"))
          .push(lbl(format!("{:.2}", self.result.power_idle.consumption)))
          .push(lbl(format!("{:.2}", self.result.power_misc.consumption)))
          .push(lbl(format!("{:.2}", self.result.power_upto_jump_drive.consumption)))
          .push(lbl(format!("{:.2}", self.result.power_upto_generator.consumption)))
          .push(lbl(format!("{:.2}", self.result.power_upto_up_down_thruster.consumption)))
          .push(lbl(format!("{:.2}", self.result.power_upto_front_back_thruster.consumption)))
          .push(lbl(format!("{:.2}", self.result.power_upto_left_right_thruster.consumption)))
          .push(lbl(format!("{:.2}", self.result.power_upto_battery.consumption)))
        )
        .push(col()
          .push(lbl("-"))
          .push(lbl("-"))
          .push(lbl("Balance"))
          .push(lbl(format!("{:.2}", self.result.power_idle.balance)))
          .push(lbl(format!("{:.2}", self.result.power_misc.balance)))
          .push(lbl(format!("{:.2}", self.result.power_upto_jump_drive.balance)))
          .push(lbl(format!("{:.2}", self.result.power_upto_generator.balance)))
          .push(lbl(format!("{:.2}", self.result.power_upto_up_down_thruster.balance)))
          .push(lbl(format!("{:.2}", self.result.power_upto_front_back_thruster.balance)))
          .push(lbl(format!("{:.2}", self.result.power_upto_left_right_thruster.balance)))
          .push(lbl(format!("{:.2}", self.result.power_upto_battery.balance)))
        )
        .push(col()
          .push(lbl("-"))
          .push(lbl("-"))
          .push(lbl("Duration: Batteries (min)"))
          .push(lbl(format!("{:.2}", self.result.power_idle.duration)))
          .push(lbl(format!("{:.2}", self.result.power_misc.duration)))
          .push(lbl(format!("{:.2}", self.result.power_upto_jump_drive.duration)))
          .push(lbl(format!("{:.2}", self.result.power_upto_generator.duration)))
          .push(lbl(format!("{:.2}", self.result.power_upto_up_down_thruster.duration)))
          .push(lbl(format!("{:.2}", self.result.power_upto_front_back_thruster.duration)))
          .push(lbl(format!("{:.2}", self.result.power_upto_left_right_thruster.duration)))
          .push(lbl(format!("{:.2}", self.result.power_upto_battery.duration)))
        )
        ;
      row.into()
    };

    let result_hydrogen: Element<_> = {
      let row = row().spacing(10)
        .push(col()
          .push(lbl("Generation (L/s)"))
          .push(lbl("Capacity: Engines (L)"))
          .push(lbl("Capacity: Tanks (L)"))
          .push(lbl("-"))
          .push(lbl("Ide (L/s)"))
          .push(lbl("Engines (L/s)"))
          .push(lbl("+Up/Down Thrusters (L/s)"))
          .push(lbl("+Front/Back Thrusters (L/s)"))
          .push(lbl("+Left/Right Thrusters (L/s)"))
        )
        .push(col()
          .push(lbl(format!("{:.2}", self.result.hydrogen_generation)))
          .push(lbl(format!("{:.2}", self.result.hydrogen_capacity_engine)))
          .push(lbl(format!("{:.2}", self.result.hydrogen_capacity_tank)))
          .push(lbl("Consumption"))
          .push(lbl(format!("{:.2}", self.result.hydrogen_idle.consumption)))
          .push(lbl(format!("{:.2}", self.result.hydrogen_engine.consumption)))
          .push(lbl(format!("{:.2}", self.result.hydrogen_upto_up_down_thruster.consumption)))
          .push(lbl(format!("{:.2}", self.result.hydrogen_upto_front_back_thruster.consumption)))
          .push(lbl(format!("{:.2}", self.result.hydrogen_upto_left_right_thruster.consumption)))
        )
        .push(col()
          .push(lbl("-"))
          .push(lbl("-"))
          .push(lbl("-"))
          .push(lbl("Balance"))
          .push(lbl(format!("{:.2}", self.result.hydrogen_idle.balance)))
          .push(lbl(format!("{:.2}", self.result.hydrogen_engine.balance)))
          .push(lbl(format!("{:.2}", self.result.hydrogen_upto_up_down_thruster.balance)))
          .push(lbl(format!("{:.2}", self.result.hydrogen_upto_front_back_thruster.balance)))
          .push(lbl(format!("{:.2}", self.result.hydrogen_upto_left_right_thruster.balance)))
        )
        .push(col()
          .push(lbl("-"))
          .push(lbl("-"))
          .push(lbl("-"))
          .push(lbl("Duration: Tanks (min)"))
          .push(lbl(format!("{:.2}", self.result.hydrogen_idle.duration)))
          .push(lbl(format!("{:.2}", self.result.hydrogen_engine.duration)))
          .push(lbl(format!("{:.2}", self.result.hydrogen_upto_up_down_thruster.duration)))
          .push(lbl(format!("{:.2}", self.result.hydrogen_upto_front_back_thruster.duration)))
          .push(lbl(format!("{:.2}", self.result.hydrogen_upto_left_right_thruster.duration)))
        )
        ;
      row.into()
    };

    let label_width = Length::Units(100);
    let value_width = Length::Units(100);
    let result = scl(&mut self.result_scrollable)
      .spacing(10)
      .padding(1)
      .push(col()
        .push(h2("Mass"))
        .push_labelled("Empty", label_width, format!("{:.0}", self.result.total_mass_empty), value_width, "kg")
        .push_labelled("Filled", label_width, format!("{:.0}", self.result.total_mass_filled), value_width, "kg")
      )
      .push(col()
        .push(h2("Volume"))
        .push_labelled("Any", label_width, format!("{:.0}", self.result.total_volume_any), value_width, "L")
        .push_labelled("Ore", label_width, format!("{:.0}", self.result.total_volume_ore), value_width, "L")
        .push_labelled("Ice", label_width, format!("{:.0}", self.result.total_volume_ice), value_width, "L")
        .push_labelled("Ore-only", label_width, format!("{:.0}", self.result.total_volume_ore_only), value_width, "L")
        .push_labelled("Ice-only", label_width, format!("{:.0}", self.result.total_volume_ice_only), value_width, "L")
      )
      .push(col()
        .push(h2("Items"))
        .push_labelled("Ore", label_width, format!("{:.0}", self.result.total_items_ore), value_width, "#")
        .push_labelled("Ice", label_width, format!("{:.0}", self.result.total_items_ice), value_width, "#")
        .push_labelled("Steel Plates", label_width, format!("{:.0}", self.result.total_items_steel_plate), value_width, "#")
      )
      .push(col()
        .push(h2("Force & Acceleration"))
        .push(result_acceleration)
      )
      .push(col()
        .push(h2("Power"))
        .push(result_power)
      )
      .push(col()
        .push(h2("Hydrogen"))
        .push(result_hydrogen)
      )
      ;

    // Root
    let root: Element<_> = col()
      .padding(10)
      .push(row()
        .align_items(Align::End)
        .push(h1("Space Engineers Calculator"))
        .push(h3(" https://github.com/Gohla/space_engineers_calc").width(Length::Fill))
      )
      .push(row()
        .spacing(20)
        .push(input)
        .push(result)
      ).into();
    root
  }
}

