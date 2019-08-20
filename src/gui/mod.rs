use std::rc::Rc;

use gio::prelude::*;
use gtk::Application;

use crate::data::Data;

use self::main_window::MainWindow;

pub mod main_window;
pub mod dialog;

pub fn run(data: Data) {
  let application = Application::new(None, Default::default()).expect("failed to initialize GTK application");
  let data = Rc::new(data);
  let main_window = MainWindow::new(data);
  application.connect_activate(move |app| {
    main_window.set_application(app);
    main_window.show();
  });
  application.run(&[]);
}
