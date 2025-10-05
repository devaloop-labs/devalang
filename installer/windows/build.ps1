# PowerShell script to build the Devalang MSI installer
# Prerequisites: WiX Toolset 3.x, 4.x, or 6.x installed

param(
    [string]$Version = "0.1.1",
    [string]$Configuration = "Release",
    [switch]$SkipBuild,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"

# Colors for messages
function Write-Info { Write-Host $args -ForegroundColor Cyan }
function Write-Success { Write-Host $args -ForegroundColor Green }
function Write-ErrorMsg { Write-Host $args -ForegroundColor Red }

# WiX Toolset detection
$wixPath = $null
$wixVersion = $null
$useWix4Plus = $false

# Check for WiX 4.0+ / 6.0+ (uses wix.exe)
$possiblePaths4Plus = @(
    "C:\Program Files\dotnet\tools",
    "C:\Program Files\WiX Toolset v6.0\bin",
    "C:\Program Files\WiX Toolset v5.0\bin",
    "C:\Program Files\WiX Toolset v4.0\bin",
    "$env:USERPROFILE\.dotnet\tools",
    "$env:ProgramFiles\dotnet\tools"
)

foreach ($path in $possiblePaths4Plus) {
    if (Test-Path "$path\wix.exe") {
        $wixPath = $path
        $useWix4Plus = $true
        $wixVersion = & "$path\wix.exe" --version 2>$null
        if (-not $wixVersion) { $wixVersion = "4.0+" }
        break
    }
}

# Check for WiX 3.x (uses candle.exe and light.exe)
if (-not $wixPath) {
    $possiblePaths3 = @(
        "C:\Program Files (x86)\WiX Toolset v3.11\bin",
        "C:\Program Files (x86)\WiX Toolset v3.14\bin",
        "$env:WIX\bin"
    )

    foreach ($path in $possiblePaths3) {
        if (Test-Path "$path\candle.exe") {
            $wixPath = $path
            $useWix4Plus = $false
            $wixVersion = "3.x"
            break
        }
    }
}

if (-not $wixPath) {
    Write-ErrorMsg "[X] WiX Toolset was not found."
    Write-Info "Download it from: https://wixtoolset.org/releases/"
    Write-Info "For WiX v4+/v6, you can also install via: dotnet tool install --global wix"
    exit 1
}

Write-Success "[OK] WiX Toolset $wixVersion found: $wixPath"

# Define paths
$rootDir = Split-Path $PSScriptRoot -Parent
$installerDir = $PSScriptRoot
$targetDir = Join-Path $rootDir "target\$Configuration"
$outputDir = Join-Path $installerDir "output"

# Cleanup if requested
if ($Clean) {
    Write-Info "[*] Cleaning temporary files..."
    Remove-Item "$installerDir\*.wixobj" -ErrorAction SilentlyContinue
    Remove-Item "$installerDir\*.wixpdb" -ErrorAction SilentlyContinue
    Remove-Item "$outputDir\*.msi" -ErrorAction SilentlyContinue
}

# Create output folder
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

# Compile Devalang if needed
if (-not $SkipBuild) {
    Write-Info "[*] Compiling Devalang..."
    Push-Location $rootDir
    try {
        cargo build --features cli --release
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo compilation failed"
        }
        Write-Success "[OK] Compilation successful"
    }
    finally {
        Pop-Location
    }
}

# Check that executable exists
$exePath = Join-Path $targetDir "devalang.exe"
if (-not (Test-Path $exePath)) {
    Write-ErrorMsg "[X] Executable was not found: $exePath"
    Write-Info "Run first: cargo build --features cli --release"
    exit 1
}

Write-Success "[OK] Executable found: $exePath"

# Build MSI installer
$outputMsi = Join-Path $outputDir "Devalang-$Version-x64.msi"

if ($useWix4Plus) {
    # WiX 4.0+ / 6.0+ - Single command
    Write-Info "[*] Building MSI with WiX $wixVersion..."
    $wixArgs = @(
        "build"
        "-nologo"
        "-arch", "x64"
        "-d", "Version=$Version"
        "-culture", "en-us"
        "-loc", "$installerDir\strings-en-us.wxl"
        "-out", $outputMsi
        "$installerDir\devalang.wxs"
    )

    & "$wixPath\wix.exe" $wixArgs
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorMsg "[X] WiX build failed (wix.exe)"
        exit 1
    }
} else {
    # WiX 3.x - Two-step process (compile + link)
    Write-Info "[*] Compiling WiX file with WiX $wixVersion..."
    $candleArgs = @(
        "-nologo"
        "-ext", "WixUIExtension"
        "-ext", "WixUtilExtension"
        "-arch", "x64"
        "-dVersion=$Version"
        "-out", "$installerDir\devalang.wixobj"
        "$installerDir\devalang.wxs"
    )

    & "$wixPath\candle.exe" $candleArgs
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorMsg "[X] WiX compilation failed (candle.exe)"
        exit 1
    }

    Write-Info "[*] Linking MSI installer..."
    $lightArgs = @(
        "-nologo"
        "-ext", "WixUIExtension"
        "-ext", "WixUtilExtension"
        "-cultures:en-us"
        "-loc", "$installerDir\strings-en-us.wxl"
        "-out", $outputMsi
        "$installerDir\devalang.wixobj"
    )

    & "$wixPath\light.exe" $lightArgs
    if ($LASTEXITCODE -ne 0) {
        Write-ErrorMsg "[X] WiX linking failed (light.exe)"
        exit 1
    }
}

Write-Success "[OK] MSI build successful"