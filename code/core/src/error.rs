use std::error::Error;

pub trait ErrorExt {
  fn into_boxed(self) -> Box<dyn Error + Send + Sync + 'static>;
}

impl<T: Error + Send + Sync + 'static> ErrorExt for T {
  fn into_boxed(self) -> Box<dyn Error + Send + Sync + 'static> {
    Box::new(self) as Box<dyn Error + Send + Sync + 'static>
  }
}
