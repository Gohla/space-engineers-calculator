use iced::{HorizontalAlignment, Length, Row, Scrollable, Text};

pub const H2_SIZE: u16 = 24;
pub const H1_SIZE: u16 = 28;


pub fn h1<L: Into<String>>(label: L) -> Text {
  Text::new(label).size(H1_SIZE)
}

pub fn h2<L: Into<String>>(label: L) -> Text {
  Text::new(label).size(H2_SIZE)
}

pub trait ScrollableExt {
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
