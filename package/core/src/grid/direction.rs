use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};

// Direction

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum Direction {
  #[default] Up,
  Down,
  Front,
  Back,
  Left,
  Right
}

impl Direction {
  #[inline]
  pub fn items() -> impl IntoIterator<Item=Self> {
    use Direction::*;
    const ITEMS: [Direction; 6] = [Up, Down, Front, Back, Left, Right];
    ITEMS.into_iter()
  }

  #[inline]
  pub const fn into_index(self) -> usize {
    use Direction::*;
    match self {
      Up => 0,
      Down => 1,
      Front => 2,
      Back => 3,
      Left => 4,
      Right => 5,
    }
  }
}

impl Display for Direction {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Direction::Up => f.write_str("Up"),
      Direction::Down => f.write_str("Down"),
      Direction::Front => f.write_str("Front"),
      Direction::Back => f.write_str("Back"),
      Direction::Left => f.write_str("Left"),
      Direction::Right => f.write_str("Right"),
    }
  }
}


// Per-direction

#[repr(transparent)]
#[derive(Default, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct PerDirection<T>([T; 6]);

impl<T> PerDirection<T> {
  #[inline]
  pub const fn get(&self, direction: Direction) -> &T { &self.0[direction.into_index()] }
  #[inline]
  pub fn get_mut(&mut self, direction: Direction) -> &mut T { &mut self.0[direction.into_index()] }

  #[inline]
  pub const fn up(&self) -> &T { self.get(Direction::Up) }
  #[inline]
  pub const fn down(&self) -> &T { self.get(Direction::Down) }
  #[inline]
  pub const fn front(&self) -> &T { self.get(Direction::Front) }
  #[inline]
  pub const fn back(&self) -> &T { self.get(Direction::Back) }
  #[inline]
  pub const fn left(&self) -> &T { self.get(Direction::Left) }
  #[inline]
  pub const fn right(&self) -> &T { self.get(Direction::Right) }

  #[inline]
  pub fn up_mut(&mut self) -> &mut T { self.get_mut(Direction::Up) }
  #[inline]
  pub fn down_mut(&mut self) -> &mut T { self.get_mut(Direction::Down) }
  #[inline]
  pub fn front_mut(&mut self) -> &mut T { self.get_mut(Direction::Front) }
  #[inline]
  pub fn back_mut(&mut self) -> &mut T { self.get_mut(Direction::Back) }
  #[inline]
  pub fn left_mut(&mut self) -> &mut T { self.get_mut(Direction::Left) }
  #[inline]
  pub fn right_mut(&mut self) -> &mut T { self.get_mut(Direction::Right) }

  #[inline]
  pub fn iter(&self) -> impl Iterator<Item=&T> { self.0.iter() }
  #[inline]
  pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut T> { self.0.iter_mut() }

  #[inline] //noinspection RsBorrowChecker
  pub fn iter_with_direction(&self) -> impl Iterator<Item=(Direction, &T)> {
    Direction::items().into_iter().map(|d| (d, &self[d]))
  }
}

impl<T> Index<Direction> for PerDirection<T> {
  type Output = T;
  #[inline]
  fn index(&self, index: Direction) -> &Self::Output {
    &self.0[index.into_index()]
  }
}

impl<T> IndexMut<Direction> for PerDirection<T> {
  #[inline]
  fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
    &mut self.0[index.into_index()]
  }
}


// Count-per-direction

pub type CountPerDirection = PerDirection<u64>;