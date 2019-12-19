use iced::{Column, HorizontalAlignment, Length, Row, Scrollable, scrollable, Text};

#[inline]
pub fn row<'a, M>() -> Row<'a, M> { Row::new().width(Length::Shrink) }

#[inline]
pub fn col<'a, M>() -> Column<'a, M> { Column::new().width(Length::Shrink) }

#[inline]
pub fn scl<M>(state: &mut scrollable::State) -> Scrollable<M> { Scrollable::new(state).width(Length::Shrink) }

pub const TXT_SIZE: u16 = 18;
pub const H3_SIZE: u16 = 20;
pub const H2_SIZE: u16 = 24;
pub const H1_SIZE: u16 = 32;

#[inline]
pub fn lbl<L: Into<String>>(label: L) -> Text { Text::new(label).size(TXT_SIZE).width(Length::Shrink) }

#[inline]
pub fn h3<L: Into<String>>(label: L) -> Text { Text::new(label).size(H3_SIZE).width(Length::Shrink) }

#[inline]
pub fn h2<L: Into<String>>(label: L) -> Text { Text::new(label).size(H2_SIZE).width(Length::Shrink) }

#[inline]
pub fn h1<L: Into<String>>(label: L) -> Text { Text::new(label).size(H1_SIZE).width(Length::Shrink) }

pub trait ColumnExt {
  fn push_labelled<L: Into<String>, V: Into<String>, U: Into<String>>(self, label: L, label_width: Length, value: V, value_width: Length, unit: U) -> Self;
}

impl<'a, Message: 'static + Clone> ColumnExt for Column<'a, Message> {
  fn push_labelled<L: Into<String>, V: Into<String>, U: Into<String>>(self, label: L, label_width: Length, value: V, value_width: Length, unit: U) -> Self {
    let text = lbl(label)
      .width(label_width);
    let value = lbl(value)
      .width(value_width)
      .horizontal_alignment(HorizontalAlignment::Right);
    let unit = lbl(unit);
    self.push(row()
      .push(text)
      .push(value)
      .push(unit)
      .padding(1)
      .spacing(2)
    )
  }
}

impl<'a, Message: 'static + Clone> ColumnExt for Scrollable<'a, Message> {
  fn push_labelled<L: Into<String>, V: Into<String>, U: Into<String>>(self, label: L, label_width: Length, value: V, value_width: Length, unit: U) -> Self {
    let text = lbl(label)
      .width(label_width);
    let value = lbl(value)
      .width(value_width)
      .horizontal_alignment(HorizontalAlignment::Right);
    let unit = lbl(unit);
    self.push(row()
      .push(text)
      .push(value)
      .push(unit)
      .padding(1)
      .spacing(2)
    )
  }
}
