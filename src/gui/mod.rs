use gio::prelude::*;
use gtk::{Application, ApplicationWindow};
use gtk::prelude::*;

use crate::data::Data;

pub mod main_window;

pub fn run(_data: &Data) {
  let application = Application::new(None, Default::default())
    .expect("failed to initialize GTK application");

  application.connect_activate(|app| {
    let main_window = main_window::MainWindow::new(app);
    main_window.show();
  });

  application.run(&[]);
}
