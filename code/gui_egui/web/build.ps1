param (
  [Boolean]$Debug = $false
)

. .\..\..\..\Common.ps1

Invoke-Wasm-Bindgen -Package "secalc_gui_egui" -Binary "secalc_gui_egui" -OutDir "code/gui_egui/web/wasm_out" -Debug $Debug
