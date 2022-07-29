use iced::{alignment, Alignment, button, Element, Length, scrollable};

use secalc_core::data::Data;
use secalc_core::grid::{Direction, GridCalculated, GridCalculator};

use crate::view::{button, col, empty, h1, h2, h3, lbl, row, scl, val};

use self::block_input::{BlockInput, BlockInputMessage};
use self::directional_block_input::{DirectionalBlockInput, DirectionalBlockInputMessage};
use self::option_input::{OptionInput, OptionInputMessage};

pub mod option_input;
pub mod block_input;
pub mod directional_block_input;

pub struct Page {
  input: Input,
  result: Result,
  result_mut: ResultMut,
  save_button_state: button::State,
  save_as_button_state: button::State,
  load_button_state: button::State,
}

pub struct Input {
  options: OptionInput,
  storage: BlockInput,
  thrust: DirectionalBlockInput,
  power: BlockInput,
  hydrogen: BlockInput,
  scrollable_state: scrollable::State,
}

pub struct Result {
  calculated: GridCalculated,
}

pub struct ResultMut {
  scrollable_state: scrollable::State,
}

#[derive(Clone, Debug)]
pub enum Message {
  InputOptionChange(OptionInputMessage),
  InputStorageChange(BlockInputMessage),
  InputThrustChange(DirectionalBlockInputMessage),
  InputPowerChange(BlockInputMessage),
  InputHydrogenChange(BlockInputMessage),
  SavePressed,
  SaveAsPressed,
  LoadPressed,
}

pub enum Action {
  CalculatorModified,
  Save,
  SaveAs,
  Load,
}

impl Page {
  pub fn new(data: &Data, default_calculator: &GridCalculator, loaded_calculator: &GridCalculator) -> Self {
    let input = {
      let options = OptionInput::new(default_calculator, loaded_calculator);
      #[cfg(not(target_arch = "wasm32"))] let label_width = Length::Units(230);
      #[cfg(target_arch = "wasm32")] let label_width = Length::Units(180);
      #[cfg(not(target_arch = "wasm32"))] let input_width = Length::Units(35);
      #[cfg(target_arch = "wasm32")] let input_width = Length::Units(30);
      let storage = {
        let mut blocks = BlockInput::new(label_width, input_width);
        blocks.add_blocks(&data, default_calculator, loaded_calculator, data.blocks.containers.values().filter(|c| c.details.store_any));
        blocks.add_blocks(&data, default_calculator, loaded_calculator, data.blocks.cockpits.values().filter(|c| c.details.has_inventory));
        blocks
      };
      let thrust = {
        #[cfg(not(target_arch = "wasm32"))] let direction_label_width = Length::Units(49);
        #[cfg(target_arch = "wasm32")] let direction_label_width = Length::Units(42);
        let mut blocks = DirectionalBlockInput::new(label_width, input_width, direction_label_width);
        blocks.add_blocks(&data, default_calculator, loaded_calculator, data.blocks.thrusters.values());
        blocks
      };
      let power = {
        let mut blocks = BlockInput::new(label_width, input_width);
        blocks.add_blocks(&data, default_calculator, loaded_calculator, data.blocks.hydrogen_engines.values());
        blocks.add_blocks(&data, default_calculator, loaded_calculator, data.blocks.reactors.values());
        blocks.add_blocks(&data, default_calculator, loaded_calculator, data.blocks.batteries.values());
        blocks
      };
      let hydrogen = {
        let mut blocks = BlockInput::new(label_width, input_width);
        blocks.add_blocks(&data, default_calculator, loaded_calculator, data.blocks.generators.values());
        blocks.add_blocks(&data, default_calculator, loaded_calculator, data.blocks.hydrogen_tanks.values());
        blocks
      };
      Input {
        options,
        storage,
        thrust,
        power,
        hydrogen,
        scrollable_state: Default::default(),
      }
    };
    let result = Result { calculated: loaded_calculator.calculate(&data) };
    let result_mut = ResultMut { scrollable_state: Default::default() };
    Self {
      input,
      result,
      result_mut,
      save_button_state: Default::default(),
      save_as_button_state: Default::default(),
      load_button_state: Default::default(),
    }
  }

  pub fn update(&mut self, message: Message, calculator: &mut GridCalculator, data: &Data) -> Option<Action> {
    let action = match message {
      Message::InputOptionChange(m) => {
        self.input.options.update(m, calculator);
        Some(Action::CalculatorModified)
      }
      Message::InputStorageChange(m) => {
        self.input.storage.update(m, calculator);
        Some(Action::CalculatorModified)
      }
      Message::InputThrustChange(m) => {
        self.input.thrust.update(m, calculator);
        Some(Action::CalculatorModified)
      }
      Message::InputPowerChange(m) => {
        self.input.power.update(m, calculator);
        Some(Action::CalculatorModified)
      }
      Message::InputHydrogenChange(m) => {
        self.input.hydrogen.update(m, calculator);
        Some(Action::CalculatorModified)
      }
      Message::SavePressed => Some(Action::Save),
      Message::SaveAsPressed => Some(Action::SaveAs),
      Message::LoadPressed => Some(Action::Load),
    };

    if let Some(Action::CalculatorModified) = &action {
      self.result.calculated = calculator.calculate(data);
    }

    action
  }

  pub fn reload_input(&mut self, calculator: &GridCalculator, data: &Data) {
    self.input.options.reload(calculator);
    self.input.storage.reload(calculator);
    self.input.thrust.reload(calculator);
    self.input.power.reload(calculator);
    self.input.hydrogen.reload(calculator);
    self.result.calculated = calculator.calculate(data);
  }

  pub fn view(&mut self) -> Element<Message> {
    let input = Self::view_input(&mut self.input);
    let result = Self::view_result(&self.result, &mut self.result_mut);
    let root: Element<_> = col()
      .spacing(10)
      .padding(10)
      .push(row()
        .align_items(Alignment::Center)
        .push(row()
          .spacing(10)
          .width(Length::Fill)
          .align_items(Alignment::Center)
          .push(h1("Space Engineers Calculator"))
          .push(button(&mut self.save_button_state, "Save").on_press(Message::SavePressed))
          .push(button(&mut self.save_as_button_state, "Save as").on_press(Message::SaveAsPressed))
          .push(button(&mut self.load_button_state, "Load").on_press(Message::LoadPressed))
        )
        .push(row()
          .width(Length::Fill)
          .push(h3("https://github.com/Gohla/space_engineers_calc").width(Length::Fill).horizontal_alignment(alignment::Horizontal::Right))
        )
      )
      .push(row()
        .spacing(10)
        .push(input)
        .push(result)
      ).into();
    root
    //.explain(iced::Color::BLACK)
  }


  fn view_input(input: &mut Input) -> Element<Message> {
    scl(&mut input.scrollable_state)
      .spacing(10)
      .padding(1)
      .push(col()
        .push(h2("Options"))
        .push(input.options.view().map(Message::InputOptionChange))
      )
      .push(col()
        .push(h2("Storage"))
        .push(input.storage.view().map(Message::InputStorageChange))
      )
      .push(col()
        .push(h2("Thrusters"))
        .push(input.thrust.view().map(Message::InputThrustChange))
      )
      .push(col()
        .push(h2("Power"))
        .push(input.power.view().map(Message::InputPowerChange))
      )
      .push(col()
        .push(h2("Hydrogen"))
        .push(input.hydrogen.view().map(Message::InputHydrogenChange))
      )
      .into()
  }


  fn view_result<'a>(result: &'a Result, result_mut: &'a mut ResultMut) -> Element<'a, Message> {
    scl(&mut result_mut.scrollable_state)
      .spacing(10)
      .padding(1)
      .push(col()
        .push(h2("Mass"))
        .push(Self::view_result_mass(&result.calculated))
      )
      .push(col()
        .push(h2("Volume"))
        .push(Self::view_result_volume(&result.calculated))
      )
      .push(col()
        .push(h2("Items"))
        .push(Self::view_result_items(&result.calculated))
      )
      .push(col()
        .push(h2("Acceleration (m/s^2)"))
        .push(Self::view_result_acceleration(&result.calculated))
      )
      .push(col()
        .push(h2("Power"))
        .push(Self::view_result_power(&result.calculated))
      )
      .push(col()
        .push(h2("Hydrogen"))
        .push(Self::view_result_hydrogen(&result.calculated))
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
          .push(lbl("Filled").horizontal_alignment(alignment::Horizontal::Center))
          .push(lbl("Gravity").horizontal_alignment(alignment::Horizontal::Center))
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
          .push(lbl("No grav.").horizontal_alignment(alignment::Horizontal::Center))
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
          .push(lbl("Empty").horizontal_alignment(alignment::Horizontal::Center))
          .push(lbl("Gravity").horizontal_alignment(alignment::Horizontal::Center))
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
          .push(lbl("No grav.").horizontal_alignment(alignment::Horizontal::Center))
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
        .push(val(format!("{:.2} min", result.power_idle.duration_battery)))
        .push(val(format!("{:.2} min", result.power_misc.duration_battery)))
        .push(val(format!("{:.2} min", result.power_upto_jump_drive.duration_battery)))
        .push(val(format!("{:.2} min", result.power_upto_generator.duration_battery)))
        .push(val(format!("{:.2} min", result.power_upto_up_down_thruster.duration_battery)))
        .push(val(format!("{:.2} min", result.power_upto_front_back_thruster.duration_battery)))
        .push(val(format!("{:.2} min", result.power_upto_left_right_thruster.duration_battery)))
        .push(val(format!("{:.2} min", result.power_upto_battery.duration_battery)))
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
