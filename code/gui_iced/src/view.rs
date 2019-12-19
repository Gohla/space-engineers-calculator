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
pub fn val<L: Into<String>>(label: L) -> Text { Text::new(label).size(TXT_SIZE).width(Length::Shrink).horizontal_alignment(HorizontalAlignment::Right) }

#[inline]
pub fn h3<L: Into<String>>(label: L) -> Text { Text::new(label).size(H3_SIZE).width(Length::Shrink) }

#[inline]
pub fn h2<L: Into<String>>(label: L) -> Text { Text::new(label).size(H2_SIZE).width(Length::Shrink) }

#[inline]
pub fn h1<L: Into<String>>(label: L) -> Text { Text::new(label).size(H1_SIZE).width(Length::Shrink) }

#[inline]
pub fn empty() -> Text { Text::new(" ").size(TXT_SIZE).width(Length::Shrink) }
