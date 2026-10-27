$ErrorActionPreference='Stop'
$binary=Join-Path $PSScriptRoot 'target\release\ak820max-assistant.exe'
$localBinary='C:\Users\pavel\Documents\ChatGPT\ajjazz\target\gpwb-assistant\release\ak820max-assistant.exe'
if(Test-Path -LiteralPath $localBinary){$binary=$localBinary}
if(-not(Test-Path -LiteralPath $binary)){throw 'Build release first'}
$destination=Join-Path $PSScriptRoot 'dist-connect'
New-Item -ItemType Directory -Force -Path $destination | Out-Null
Copy-Item -LiteralPath $binary -Destination (Join-Path $destination 'AJAZZ-Connect.exe') -Force
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'README-PORTABLE.md') -Destination (Join-Path $destination 'README.md') -Force
foreach($name in @('LICENSE','STATUS.json','THIRD-PARTY.md','THIRD-PARTY-LICENSES.txt')){Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination $destination -Force}
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'docs\KEYBOARD-FAMILIES-RE.md') -Destination $destination -Force
$hashes=Get-ChildItem -LiteralPath $destination -File | Where-Object {$_.Name -ne 'SHA256SUMS.txt'} | ForEach-Object {
    "$((Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLower())  $($_.Name)"
}
[IO.File]::WriteAllLines((Join-Path $destination 'SHA256SUMS.txt'),$hashes)
$archive=Join-Path (Split-Path -Parent $PSScriptRoot) 'AJAZZ-Connect-0.2.0.zip'
Compress-Archive -Path (Join-Path $destination '*') -DestinationPath $archive -Force
Get-Item -LiteralPath $archive | Select-Object FullName,Length
