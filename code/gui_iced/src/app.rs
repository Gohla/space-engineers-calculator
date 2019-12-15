use std::fmt::Debug;

use iced::{Application, Color, Command, Element, Row, scrollable, Scrollable, Text};

use secalc_core::grid::GridCalculator;

use crate::data_bind::{DataBind, DataBindMessage};

macro_rules! create_input_options {
  ($($field:ident, $type:ty, $message:ident, $name:expr, $format:expr);*) => {
    struct InputOptions {
      $($field: DataBind<$type>,)*
    }

    impl InputOptions {
      fn new(calc: &GridCalculator) -> Self {
        Self {
          $($field: DataBind::new($name, calc.$field, format!($format, calc.$field)),)*
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

create_input_options!(
  gravity_multiplier, f64, GravityMultiplier, "Gravity Multiplier", "{:.1}";
  container_multiplier, f64, ContainerMultiplier, "Container Multiplier", "{:.1}";
  planetary_influence, f64, PlanetaryInfluence, "Planetary Influence", "{:.1}";
  additional_mass, f64, AdditionalMass, "Additional Mass", "{:.1}";
  ice_only_fill, f64, IceOnlyFill, "Ice-only-fill", "{:.1}";
  ore_only_fill, f64, OreOnlyFill, "Ore-only-fill", "{:.1}";
  any_fill_with_ice, f64, AnyFillWithIce, "Any-fill with Ice", "{:.1}";
  any_fill_with_ore, f64, AnyFillWithOre, "Any-fill with Ore", "{:.1}";
  any_fill_with_steel_plates, f64, AnyFillWithSteelPlates, "Any-fill with Steel Plates", "{:.1}"
);

pub struct App {
  input_scrollable: scrollable::State,
  result_scrollable: scrollable::State,
  input_options: InputOptions,
  calc: GridCalculator,
}

impl Default for App {
  fn default() -> Self {
    let calc = GridCalculator::default();
    Self {
      input_scrollable: scrollable::State::default(),
      result_scrollable: scrollable::State::default(),
      input_options: InputOptions::new(&calc),
      calc,
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
      Message::InputOptionChange(m) => self.input_options.update(m, &mut self.calc),
    }
    Command::none()
  }

  fn view(&mut self) -> Element<Message> {
    let heading_size = 28;
    let input = Scrollable::new(&mut self.input_scrollable)
      .spacing(2)
      .padding(2);

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
      .spacing(2)
      .padding(2)
      .push(Text::new("Mass").size(heading_size))
      .push(Text::new("Volume").size(heading_size))
      .push(Text::new("Items").size(heading_size))
      .push(Text::new("Force & Acceleration").size(heading_size))
      .push(Text::new("Power").size(heading_size))
      .push(Text::new("Hydrogen").size(heading_size))
      .into();

    let content: Element<_> = Row::new()
      .spacing(10)
      .padding(10)
      .push(input)
      .push(result)
      .into();
    content.explain(Color::from_rgb(1.0, 0.0, 0.0))
  }
}
