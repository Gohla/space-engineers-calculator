function Invoke-Wasm-Bindgen {
  param (
    [String]$Package,
    [String]$OutDir,
    [String]$Binary = $null,
    [Boolean]$Debug = $false
  )

  Push-Location -Path "$PSScriptRoot"
  try {
    $BinPart = if($Binary -ne $null) { "--bin $Binary" } else { "" }
    $ReleasePart = if($Debug) { "" } else { "--release" }
    Invoke-Expression "cargo build --package $Package $BinPart --target wasm32-unknown-unknown --target-dir target-wasm $ReleasePart"
    
    $TargetSubdir = if($Debug) { "debug" } else { "release" }
    $WasmFileName = if($Binary -ne $null) { $Binary } else { $Package }
    Invoke-Expression "wasm-bindgen --out-dir $OutDir --target web --no-typescript target-wasm/wasm32-unknown-unknown/$TargetSubdir/$WasmFileName.wasm"
  } finally {
    Pop-Location
  }
}
