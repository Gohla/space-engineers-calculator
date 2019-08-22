use std::error::Error;
use std::path::{Path, PathBuf};

use gtk::{ButtonsType, DialogFlags, FileChooserAction, FileChooserNative, IsA, MessageDialog, MessageType, Window, FileFilter};
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
    chooser.set_do_overwrite_confirmation(true);
    let filter = FileFilter::new();
    filter.add_pattern("*.secalc");
    chooser.add_filter(&filter);
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


pub struct ErrorDialog {
  message_dialog: MessageDialog,
}

impl ErrorDialog {
  pub fn new<W: IsA<Window>>(parent: &W, text: &str) -> Self {
    let message_dialog = MessageDialog::new(Some(parent), DialogFlags::MODAL, MessageType::Error, ButtonsType::Ok, text);
    Self { message_dialog }
  }

  pub fn from_error_and_run<W: IsA<Window>, E: Error>(parent: &W, error: E) {
    let error_dialog = Self::new(parent, &format!("{}", error));
    error_dialog.run();
  }

  pub fn from_result_and_run<W: IsA<Window>, T, E: Error>(parent: &W, result: Result<T, E>) {
    if let Err(error) = result {
      Self::from_error_and_run(parent, error);
    }
  }

  pub fn run(&self) {
    self.message_dialog.run();
  }
}

impl Drop for ErrorDialog {
  fn drop(&mut self) {
    self.message_dialog.destroy();
  }
}


pub trait ErrorDialogResultExt {
  fn show_error_as_dialog<W: IsA<Window>>(self, parent: &W);
}

impl<T, E: Error> ErrorDialogResultExt for Result<T, E> {
  fn show_error_as_dialog<W: IsA<Window>>(self, parent: &W) {
    ErrorDialog::from_result_and_run(parent, self);
  }
}