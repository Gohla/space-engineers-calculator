#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

use crate::app::App;

mod app;

fn main() {
  // Setup panics to log to the console on WASM.
  #[cfg(target_arch = "wasm32")] {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
  }

  // Setup environment variables from .env on native.
  #[cfg(not(target_arch = "wasm32"))] {
    dotenv::dotenv().ok();
  }

  // Setup tracing.
  let layered = tracing_subscriber::registry();
  #[cfg(not(target_arch = "wasm32"))] {
    layered
      .with(
        tracing_subscriber::fmt::layer()
          .with_writer(std::io::stderr)
          .with_filter(EnvFilter::from_env("LOG"))
      )
      .init();
  }
  #[cfg(target_arch = "wasm32")] {
    layered
      .with(main_filter_layer)
      .with(tracing_wasm::WASMLayer::new(tracing_wasm::WASMLayerConfig::default()))
      .init();
  }

  // Run application on native.
  #[cfg(not(target_arch = "wasm32"))] {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
      "Space Engineers Calculator",
      native_options,
      Box::new(|ctx| Box::new(App::new(ctx))),
    );
  }

  // Run application on WASM.
  #[cfg(target_arch = "wasm32")] {
    eframe::start_web(canvas_id, Box::new(|ctx| Box::new(App::new(cc))))
  }
}