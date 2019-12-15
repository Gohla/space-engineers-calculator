use std::fmt::Debug;

use iced::{Application, Color, Command, Element, Length, Row, scrollable, Scrollable, Text};

use secalc_core::data::Data;
use secalc_core::grid::{GridCalculated, GridCalculator};

use crate::data_bind::{DataBind, DataBindMessage};
use crate::util::Msg;

// Input options

macro_rules! create_input_options {
  ($label_length:expr; $input_length:expr; $($field:ident, $type:ty, $message:ident, $label:expr, $format:expr);*) => {
    struct InputOptions {
      $($field: DataBind<$type>,)*
    }

    impl InputOptions {
      fn new(calc: &GridCalculator) -> Self {
        Self {
          $($field: DataBind::new($label, $label_length, calc.$field, format!($format, calc.$field), $input_length),)*
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

      pub fn view<M: 'static + Clone + Debug>(&mut self, on_change: impl 'static + Copy + Fn(InputOptionMessage) -> M) -> impl IntoIterator<Item=Element<M>> {
        vec![
          $(self.$field.view(move |s| on_change(InputOptionMessage::$message(s))),)*
        ]
      }
    }
  }
}

create_input_options!(Length::Units(230); Length::Units(100);
  gravity_multiplier, f64, GravityMultiplier, "Gravity Multiplier", "{:.1}";
  container_multiplier, f64, ContainerMultiplier, "Container Multiplier", "{:.1}";
  planetary_influence, f64, PlanetaryInfluence, "Planetary Influence", "{:.1}";
  additional_mass, f64, AdditionalMass, "Additional Mass (kg)", "{}";
  ice_only_fill, f64, IceOnlyFill, "Ice-only-fill (%)", "{:.1}";
  ore_only_fill, f64, OreOnlyFill, "Ore-only-fill (%)", "{:.1}";
  any_fill_with_ice, f64, AnyFillWithIce, "Any-fill with Ice (%)", "{:.1}";
  any_fill_with_ore, f64, AnyFillWithOre, "Any-fill with Ore (%)", "{:.1}";
  any_fill_with_steel_plates, f64, AnyFillWithSteelPlates, "Any-fill with Steel Plates (%)", "{:.1}"
);

// Application

pub struct App {
  input_options: InputOptions,

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

    Self {
      input_options: InputOptions::new(&calculator),
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
    }
    self.result = self.calculator.calculate(&self.data);
    Command::none()
  }

  fn view(&mut self) -> Element<Message> {
    let heading_size = 28;
    let input = Scrollable::new(&mut self.input_scrollable)
      .spacing(1)
      .padding(0);

    // Options
    let mut input = input
      .push(Text::new("Options").size(heading_size));
    for elem in self.input_options.view(Message::InputOptionChange) {
      input = input.push(elem);
    }

    let input: Element<_> = input
      .push(Text::new("Storage").size(heading_size))
      .push(Text::new("Thrusters").size(heading_size))
      .push(Text::new("Power").size(heading_size))
      .push(Text::new("Hydrogen").size(heading_size))
      .into();

    let result: Element<_> = Scrollable::new(&mut self.result_scrollable)
      .spacing(1)
      .padding(0)
      .push(Text::new("Mass").size(heading_size))
      .push_label_value("Empty (kg)", format!("{:.2}", self.result.total_mass_empty))
      .push_label_value("Filled (kg)", format!("{:.2}", self.result.total_mass_filled))
      .push(Text::new("Volume").size(heading_size))
      .push_label_value("Any (L)", format!("{:.2}", self.result.total_volume_any))
      .push_label_value("Ore (L)", format!("{:.2}", self.result.total_volume_ore))
      .push_label_value("Ice (L)", format!("{:.2}", self.result.total_volume_ice))
      .push_label_value("Ore-only (L)", format!("{:.2}", self.result.total_volume_ore_only))
      .push_label_value("Ice-only (L)", format!("{:.2}", self.result.total_volume_ice_only))
      .push(Text::new("Items").size(heading_size))
      .push_label_value("Ore (#)", format!("{:.2}", self.result.total_items_ore))
      .push_label_value("Ice (#)", format!("{:.2}", self.result.total_items_ice))
      .push_label_value("Steel Plates (#)", format!("{:.2}", self.result.total_items_steel_plate))
      .push(Text::new("Force & Acceleration").size(heading_size))
      .push(Text::new("Power").size(heading_size))
      .push(Text::new("Hydrogen").size(heading_size))
      .into();

    let content: Element<_> = Row::new()
      .spacing(10)
      .padding(5)
      .push(input)
      .push(result)
      .into();
    content.explain(Color::from_rgb(1.0, 0.0, 0.0))
  }
}

trait ScrollableExt {
  fn push_label_value<L: Into<String>, V: Into<String>>(self, label: L, value: V) -> Self;
}

impl<'a, Message: Msg> ScrollableExt for Scrollable<'a, Message> {
  fn push_label_value<L: Into<String>, V: Into<String>>(self, label: L, value: V) -> Self {
    return self
      .push(Row::new()
        .push(Text::new(label))
        .push(Text::new(value))
      )
  }
}
