use gio::prelude::*;
use gtk::{Application, ApplicationWindow, Entry, Label};
use gtk::prelude::*;

pub struct MainWindow {
  window: ApplicationWindow,

  gravity_multiplier: Entry,
  container_multiplier: Entry,
  planetary_influence: Entry,
  inventory_fill: Entry,
  additional_mass: Entry,

  total_volume_ore: Label,

}

impl MainWindow {
  pub fn new(app: &Application) -> Self {
    let glade_src = include_str!("main_window.glade");
    let builder = gtk::Builder::new_from_string(glade_src);
    let window: ApplicationWindow = builder.get_object("application_window").unwrap();
    window.set_application(Some(app));

    let gravity_multiplier = builder.get_object("gravity_multiplier").unwrap();
    let container_multiplier = builder.get_object("container_multiplier").unwrap();
    let planetary_influence = builder.get_object("planetary_influence").unwrap();
    let inventory_fill = builder.get_object("inventory_fill").unwrap();
    let additional_mass = builder.get_object("additional_mass").unwrap();

    let total_volume_ore = builder.get_object("total_volume_ore").unwrap();

    MainWindow {
      window,
      gravity_multiplier,
      container_multiplier,
      planetary_influence,
      inventory_fill,
      additional_mass,
      total_volume_ore
    }
  }

  pub fn show(&self) {
    self.window.show_all();
  }
}
