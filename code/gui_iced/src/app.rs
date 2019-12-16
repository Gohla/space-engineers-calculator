use std::fmt::Debug;

use iced::{Application, Color, Column, Command, Element, HorizontalAlignment, Length, Row, scrollable, Scrollable, Text};
use linked_hash_map::LinkedHashMap;

use secalc_core::data::blocks::{BlockId, Blocks, GridSize};
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

#[derive(Default)]
struct SmallAndLarge<T> {
  pub small: T,
  pub large: T,
}

impl<T> SmallAndLarge<T> {
  pub fn for_size(&mut self, size: GridSize) -> &mut T {
    match size { GridSize::Small => &mut self.small, GridSize::Large => &mut self.large }
  }
}


// Application

pub struct App {
  input_options: InputOptions,
  input_storage: SmallAndLarge<LinkedHashMap<BlockId, DataBind<u64>>>,

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

    let (small_containers, large_containers) = Blocks::small_and_large_sorted(data.blocks.containers.values().filter(|c| c.details.store_any));
    let mut input_storage = SmallAndLarge::<LinkedHashMap<BlockId, DataBind<u64>>>::default();
    for block in small_containers {
      input_storage.small.insert(block.id.clone(), DataBind::new(block.name(&data.localization), Length::Units(250), 0, "0", Length::Units(35), "#"));
    }
    for block in large_containers {
      input_storage.large.insert(block.id.clone(), DataBind::new(block.name(&data.localization), Length::Units(250), 0, "0", Length::Units(35), "#"));
    }

    Self {
      input_options,
      input_storage,
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
  InputStorageChange(BlockId, GridSize, DataBindMessage),
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
      Message::InputStorageChange(id, size, m) => {
        if let Some(data_bind) = self.input_storage.for_size(size).get_mut(&id) {
          data_bind.update(m, self.calculator.containers.entry(id.clone()).or_default())
        }
      }
    }
    self.result = self.calculator.calculate(&self.data);
    Command::none()
  }

  fn view(&mut self) -> Element<Message> {
    let input = Scrollable::new(&mut self.input_scrollable)
      .spacing(1)
      .padding(0);

    // Options
    let mut input = input
      .push(h1("Options"));
    for elem in self.input_options.view() {
      input = input.push(elem.map(Message::InputOptionChange));
    }

    // Storage
    let input = input
      .push(h1("Storage"));
    let SmallAndLarge { small: ref mut storage_small, large: ref mut storage_large } = self.input_storage;
    // Small
    let mut input_storage_small = Column::new()
      .push(h2("Small grid"));
    for (id, data_bind) in storage_small {
      let id = id.clone(); // Clone to simplify lifetimes: we have an owned String now.
      input_storage_small = input_storage_small.push(data_bind.view().map(move |m| Message::InputStorageChange(
        // Clone again because this is a Fn closure that is callable multiple times: each call needs a separate clone.
        id.clone(),
        GridSize::Small,
        m
      )))
    }
    // Large
    let mut input_storage_large = Column::new()
      .push(h2("Large grid"));
    for (id, data_bind) in storage_large {
      let id = id.clone(); // Clone to simplify lifetimes: we have an owned String now.
      input_storage_large = input_storage_large.push(data_bind.view().map(move |m| Message::InputStorageChange(
        // Clone again because this is a Fn closure that is callable multiple times: each call needs a separate clone.
        id.clone(),
        GridSize::Large,
        m
      )))
    }
    let input = input.push(Row::new()
      .spacing(2)
      .padding(0)
      .push(input_storage_small)
      .push(input_storage_large)
    );

    let input = input
      .push(h1("Thrusters"))
      .push(h1("Power"))
      .push(h1("Hydrogen"))
      ;

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
