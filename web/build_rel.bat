cargo build --manifest-path=../Cargo.toml --package secalc_gui_iced --bin secalc_gui_iced --target wasm32-unknown-unknown --release
wasm-bindgen ..\target\wasm32-unknown-unknown\release\secalc_gui_iced.wasm --out-dir . --web
