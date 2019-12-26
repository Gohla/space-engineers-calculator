# Space Engineers Calculator

Calculator for designing Space Engineers ships. A live version of this calculator can be found at https://secalc.gohla.nl/.

## Requirements

* Rust - [installation instructions](https://www.rust-lang.org/tools/install)

## Running

### Windows/Linux/macOS

Run the native calculator GUI with `cargo run --bin secalc_gui_iced`.

### Web

Compile the web calculator GUI with:

```shell script
cargo build --package secalc_gui_iced --bin secalc_gui_iced --target wasm32-unknown-unknown
wasm-bindgen target\wasm32-unknown-unknown\debug\secalc_gui_iced.wasm --out-dir web --web
```

which will result in the `web/secalc_gui_iced.js` and `web/secalc_gui_iced_bg.wasm` files, which are combined in the `web/index.html` file.
A web server is needed as loading WebAssembly files from `file://` links is not supported.
If you have Python installed, you can run `python -m serve` in the `web` directory to start a simple web server, which can be accessed from `http://127.0.0.1:1337/`.
