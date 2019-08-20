use std::path::{Path, PathBuf};

use gio::prelude::*;
use gtk::{FileChooserAction, FileChooserNative, IsA, Window};
use gtk::prelude::*;

pub struct FileDialog {
  chooser: FileChooserNative,
}

impl FileDialog {
  pub fn new<W: IsA<Window>, P: AsRef<Path>>(
    title: &str,
    parent: &W,
    action: FileChooserAction,
    accept_label: &str,
    cancel_label: &str,
    current_dir_path: Option<P>,
    current_file_path: Option<P>,
  ) -> Self {
    let chooser = FileChooserNative::new(Some(title), Some(parent), action, Some(accept_label), Some(cancel_label));
    if let Some(current_dir) = current_dir_path {
      chooser.set_current_folder(current_dir);
    }
    if let Some(current_file) = current_file_path {
      chooser.set_current_name(current_file)
    }
    FileDialog { chooser }
  }

  pub fn new_open<W: IsA<Window>, P: AsRef<Path>>(
    parent: &W,
    current_dir_path: Option<P>,
  ) -> Self {
    Self::new("Open", parent, FileChooserAction::Open, "Open", "Cancel", current_dir_path, None)
  }

  pub fn new_save<W: IsA<Window>, P: AsRef<Path>>(
    parent: &W,
    current_dir_path: Option<P>,
    current_file_path: Option<P>,
  ) -> Self {
    Self::new("Open", parent, FileChooserAction::Save, "Save", "Cancel", current_dir_path, current_file_path)
  }


  pub fn run(&self) -> Option<PathBuf> {
    if self.chooser.run() == gtk_sys::GTK_RESPONSE_ACCEPT {
      if let Some(path) = self.chooser.get_filename() {
        return Some(path)
      }
    }
    None
  }
}

impl Drop for FileDialog {
  fn drop(&mut self) {
    self.chooser.destroy();
  }
}
