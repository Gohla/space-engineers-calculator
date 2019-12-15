use std::fmt::Debug;

pub trait Msg = 'static + Clone + Debug;
