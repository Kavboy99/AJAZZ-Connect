$ErrorActionPreference='Stop'
$destination=Join-Path $PSScriptRoot 'source-dist'
New-Item -ItemType Directory -Force -Path $destination | Out-Null
$files=@('Cargo.toml','Cargo.lock','README.md','README-PORTABLE.md','LICENSE','STATUS.json','THIRD-PARTY.md','THIRD-PARTY-LICENSES.txt','build.ps1','package.ps1','package-source.ps1','.gitignore')
foreach($directory in @('src','tests','docs','tools')) {
    $files+=Get-ChildItem -LiteralPath (Join-Path $PSScriptRoot $directory) -File -Recurse |
        Where-Object {$_.Extension -notin @('.exe','.pyc')} |
        ForEach-Object {[IO.Path]::GetRelativePath($PSScriptRoot,$_.FullName)}
}
$files+=@('research/SOURCE-MANIFEST.json','research/AK820MAX-V1-BYTECH.json',
    'research/sources/BakulBd__OpenKey/fixtures/rgb/sonix-feature-lighting.json',
    'research/sources/BakulBd__OpenKey/LICENSE',
    'research/sources/microsoft__ArduinoHidForWindows/src/LampArrayReportDescriptor.h',
    'research/sources/microsoft__ArduinoHidForWindows/LICENSE')
$files+=Get-ChildItem -LiteralPath (Join-Path $PSScriptRoot 'research/keyboard-updater') -File -Filter '*.json' |
    ForEach-Object {[IO.Path]::GetRelativePath($PSScriptRoot,$_.FullName)}
foreach($relative in $files) {
    $source=Join-Path $PSScriptRoot $relative
    $target=Join-Path $destination $relative
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $target) | Out-Null
    Copy-Item -LiteralPath $source -Destination $target -Force
}
$archive=Join-Path (Split-Path -Parent $PSScriptRoot) 'AJAZZ-Connect-0.2.0-source.zip'
Compress-Archive -Path (Join-Path $destination '*') -DestinationPath $archive -Force
Get-Item -LiteralPath $archive | Select-Object FullName,Length

