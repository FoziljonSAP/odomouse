# Builds Odomouse for Windows into windows\dist\.
#   powershell -ExecutionPolicy Bypass -File windows\build.ps1
# Needs: Rust (rustup, MSVC toolchain), .NET SDK 6+ (only for building;
# the app itself runs on the .NET Framework 4.8 that ships with Windows).
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$root = Split-Path -Parent $here
$core = Join-Path $root 'core'
$proj = Join-Path $here 'Odomouse'
$native = Join-Path $proj 'native'

# Link the C runtime statically: odomouse_core.dll then runs on PCs without
# the Visual C++ Redistributable.
$env:RUSTFLAGS = '-C target-feature=+crt-static'

$targets = @{ 'x64' = 'x86_64-pc-windows-msvc'; 'x86' = 'i686-pc-windows-msvc' }
foreach ($arch in $targets.Keys) {
    $t = $targets[$arch]
    rustup target add $t | Out-Null
    cargo build --manifest-path (Join-Path $core 'Cargo.toml') --release --target $t
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed for $t" }
    New-Item -ItemType Directory -Force -Path (Join-Path $native $arch) | Out-Null
    Copy-Item (Join-Path $core "target\$t\release\odomouse_core.dll") (Join-Path $native "$arch\odomouse_core.dll") -Force
}

dotnet build (Join-Path $proj 'Odomouse.csproj') -c Release -o (Join-Path $here 'build\app')
if ($LASTEXITCODE -ne 0) { throw 'dotnet build failed' }

$dist = Join-Path $here 'dist'
New-Item -ItemType Directory -Force -Path $dist | Out-Null
Write-Host "Ilova: $(Join-Path $here 'build\app')  (Odomouse.exe)"

# ---- installer (Inno Setup 6): dist\Odomouse-Setup.exe
$iscc = @(
    (Get-Command iscc.exe -ErrorAction SilentlyContinue | ForEach-Object Source),
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe",
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe"
) | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $iscc) {
    Write-Host "Inno Setup 6 topilmadi, o'rnatuvchi yig'ilmadi (https://jrsoftware.org/isdl.php)."
    exit 0
}
$version = ([xml](Get-Content (Join-Path $proj 'Odomouse.csproj'))).Project.PropertyGroup.Version | Select-Object -First 1
& $iscc /Q "/DAppVersion=$version" (Join-Path $here 'installer.iss')
if ($LASTEXITCODE -ne 0) { throw 'Inno Setup failed' }
Write-Host "O'rnatuvchi: $(Join-Path $dist 'Odomouse-Setup.exe')"
