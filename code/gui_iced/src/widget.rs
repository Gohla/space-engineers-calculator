use iced::{Column, HorizontalAlignment, Length, Row, Scrollable, scrollable, Text};

#[inline]
pub fn row<'a, M>() -> Row<'a, M> { Row::new().width(Length::Shrink) }

#[inline]
pub fn col<'a, M>() -> Column<'a, M> { Column::new().width(Length::Shrink) }

#[inline]
pub fn scl<M>(state: &mut scrollable::State) -> Scrollable<M> { Scrollable::new(state).width(Length::Shrink) }

#[inline]
pub fn txt<L: Into<String>>(label: L) -> Text { Text::new(label).width(Length::Shrink).size(16) }

#[inline]
pub fn h1<L: Into<String>>(label: L) -> Text {
  Text::new(label).size(24)
}

#[inline]
pub fn h2<L: Into<String>>(label: L) -> Text {
  Text::new(label).size(20)
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
