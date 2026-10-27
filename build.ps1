param([ValidateSet('test','check','release','clippy','fmt')][string]$Action='test')
$ErrorActionPreference='Stop'
$toolRoot='C:\Users\pavel\Documents\ChatGPT\ajjazz'
$cargoBinary=Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
$rustcBinary=Join-Path $env:USERPROFILE '.cargo\bin\rustc.exe'
$savedPath=$env:PATH
$savedFlags=$env:CARGO_ENCODED_RUSTFLAGS
$savedLibraries=$env:LIBRARY_PATH
$savedTarget=$env:CARGO_TARGET_DIR
Push-Location -LiteralPath $PSScriptRoot
try {
    if(Test-Path -LiteralPath $toolRoot) {$env:CARGO_TARGET_DIR=Join-Path $toolRoot 'target\gpwb-assistant'}
    $sysroot=& $rustcBinary --print sysroot
    $selfContained=Join-Path $sysroot 'lib\rustlib\x86_64-pc-windows-gnu\bin\self-contained'
    $w64=Join-Path $toolRoot '.tools\w64devkit\bin'
    if((Test-Path -LiteralPath $selfContained) -and (Test-Path -LiteralPath $w64)) {
        $env:PATH="$w64;$env:PATH"
        $env:CARGO_ENCODED_RUSTFLAGS="-C$([char]31)link-self-contained=yes$([char]31)-C$([char]31)dlltool=$(Join-Path $w64 'dlltool.exe')"
        $env:LIBRARY_PATH=Join-Path $toolRoot '.tools\gnu-extra'
    }
    switch($Action) {
        'release' {& $cargoBinary build --release}
        'clippy' {& $cargoBinary clippy --all-targets -- -D warnings}
        'fmt' {& $cargoBinary fmt --all -- --check}
        default {& $cargoBinary $Action}
    }
    if($LASTEXITCODE -ne 0){throw "Build action failed: $Action"}
} finally {
    Pop-Location
    $env:PATH=$savedPath;$env:CARGO_ENCODED_RUSTFLAGS=$savedFlags;$env:LIBRARY_PATH=$savedLibraries;$env:CARGO_TARGET_DIR=$savedTarget
}
