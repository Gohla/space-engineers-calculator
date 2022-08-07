param (
  [Boolean]$Debug = $false
)

Push-Location -Path "$PSScriptRoot"
try {
  $ReleasePart = if($Debug) { "" } else { "--release" }
  Invoke-Expression "cargo +nightly build --package secalc_gui --target wasm32-unknown-unknown --target-dir target-wasm $ReleasePart"
  $TargetSubdir = if($Debug) { "debug" } else { "release" }
  Invoke-Expression "wasm-bindgen --out-dir code/gui/web/wasm_out --target web --no-typescript target-wasm/wasm32-unknown-unknown/$TargetSubdir/secalc_gui.wasm"
} finally {
  Pop-Location
}
