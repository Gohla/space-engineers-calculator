use std::error::Error;

pub trait ErrorExt {
  fn into_boxed(self) -> Box<dyn Error>;
}

impl<T: Error + 'static> ErrorExt for T {
  fn into_boxed(self) -> Box<dyn Error> {
    Box::new(self) as Box<dyn Error>
  }
}
