[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [string]$OutputDir = (Join-Path $PSScriptRoot "dist\msi"),

    [Parameter(Mandatory = $false)]
    [string]$PackageVersion,

    [Parameter(Mandatory = $false)]
    [switch]$SkipBuild,

    [Parameter(Mandatory = $false)]
    [string]$PresentMonVersion = "2.4.1",

    [Parameter(Mandatory = $false)]
    [switch]$InstallWix
)

$ErrorActionPreference = "Stop"

$ProjectRoot = $PSScriptRoot
$CargoProject = Join-Path $ProjectRoot "tui-rs"
$CargoToml = Join-Path $CargoProject "Cargo.toml"
$ReleaseExe = Join-Path $CargoProject "target\release\chaosgamemode.exe"
$Wxs = Join-Path $ProjectRoot "packaging\wix\chaosgamemode.wxs"
$PayloadDir = Join-Path $OutputDir "payload"
$ToolsDir = Join-Path $ProjectRoot ".tools"
$LocalDotnetDir = Join-Path $ToolsDir "dotnet"
$LocalWixDir = Join-Path $ToolsDir "wix"
$PresentMonCacheDir = Join-Path $ToolsDir "presentmon"
$Wix4Version = "4.0.6"

function Write-Step {
    param([string]$Message)
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Get-CargoVersion {
    $contents = Get-Content -Raw -Path $CargoToml
    $match = [regex]::Match($contents, '(?m)^\s*version\s*=\s*"([^"]+)"')
    if (-not $match.Success) {
        throw "No se pudo leer version desde $CargoToml"
    }

    return $match.Groups[1].Value
}

function Get-Wix3Toolset {
    $candle = Get-Command candle.exe -ErrorAction SilentlyContinue
    $light = Get-Command light.exe -ErrorAction SilentlyContinue
    if ($candle -and $light) {
        return [pscustomobject]@{
            Candle = $candle.Source
            Light = $light.Source
        }
    }

    $candidates = @(
        "${env:ProgramFiles(x86)}\WiX Toolset v3.14\bin",
        "${env:ProgramFiles(x86)}\WiX Toolset v3.11\bin",
        "$env:ProgramFiles\WiX Toolset v3.14\bin",
        "$env:ProgramFiles\WiX Toolset v3.11\bin"
    )

    foreach ($dir in $candidates) {
        if ([string]::IsNullOrWhiteSpace($dir)) {
            continue
        }

        $candidateCandle = Join-Path $dir "candle.exe"
        $candidateLight = Join-Path $dir "light.exe"
        if ((Test-Path $candidateCandle) -and (Test-Path $candidateLight)) {
            return [pscustomobject]@{
                Candle = $candidateCandle
                Light = $candidateLight
            }
        }
    }

    return $null
}

function Get-DotNetCommand {
    $dotnet = Get-Command dotnet.exe -ErrorAction SilentlyContinue
    if ($dotnet) {
        return $dotnet.Source
    }

    $localDotnet = Join-Path $LocalDotnetDir "dotnet.exe"
    if (Test-Path $localDotnet) {
        return $localDotnet
    }

    return $null
}

function Get-Wix4Toolset {
    $localWix = Join-Path $LocalWixDir "wix.exe"
    if (Test-Path $localWix) {
        return $localWix
    }

    $wix = Get-Command wix.exe -ErrorAction SilentlyContinue
    if ($wix) {
        return $wix.Source
    }

    return $null
}

function Install-Wix3Toolset {
    $winget = Get-Command winget -ErrorAction SilentlyContinue
    if (-not $winget) {
        throw "WiX Toolset no encontrado y winget no esta disponible. Instala WiX Toolset v3 y vuelve a ejecutar build-msi.ps1."
    }

    Write-Step "Instalando WiX Toolset con winget"
    & winget install --id WiXToolset.WiXToolset --exact --accept-package-agreements --accept-source-agreements
}

function Install-LocalDotNetSdk {
    $dotnet = Get-DotNetCommand
    if ($dotnet) {
        return $dotnet
    }

    New-Item -ItemType Directory -Path $ToolsDir -Force | Out-Null
    $installScript = Join-Path $ToolsDir "dotnet-install.ps1"
    if (-not (Test-Path $installScript)) {
        Write-Step "Descargando dotnet-install.ps1"
        Invoke-WebRequest -Uri "https://dot.net/v1/dotnet-install.ps1" -OutFile $installScript
    }

    Write-Step "Instalando .NET SDK local en .tools\dotnet"
    & $installScript -Channel 8.0 -InstallDir $LocalDotnetDir

    $dotnet = Get-DotNetCommand
    if (-not $dotnet) {
        throw "No se pudo preparar .NET SDK local para WiX v4."
    }

    return $dotnet
}

function Install-Wix4Toolset {
    $dotnet = Install-LocalDotNetSdk
    New-Item -ItemType Directory -Path $ToolsDir -Force | Out-Null

    if (-not (Test-Path (Join-Path $LocalWixDir "wix.exe"))) {
        Write-Step "Instalando WiX v4 local en .tools\wix"
        & $dotnet tool install --tool-path $LocalWixDir wix --version $Wix4Version
    }
}

function Invoke-Wix4 {
    param(
        [string]$WixExe,
        [string[]]$Arguments
    )

    $dotnet = Get-DotNetCommand
    if ($dotnet) {
        $dotnetRoot = Split-Path -Parent $dotnet
        $env:DOTNET_ROOT = $dotnetRoot
        $env:PATH = "$dotnetRoot;$env:PATH"
    }

    & $WixExe @Arguments
}

function Get-InstalledPresentMonConsolePath {
    foreach ($commandName in @("presentmon", "presentmon.exe", "PresentMon.exe")) {
        $command = Get-Command $commandName -ErrorAction SilentlyContinue
        if ($command -and (Test-Path $command.Source)) {
            return $command.Source
        }
    }

    $wingetPackageDir = Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages\Intel.PresentMon.Console_Microsoft.Winget.Source_8wekyb3d8bbwe"
    $direct = Join-Path $wingetPackageDir "presentmon.exe"
    if (Test-Path $direct) {
        return $direct
    }

    if (Test-Path $wingetPackageDir) {
        $found = Get-ChildItem -Path $wingetPackageDir -Filter "presentmon.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($found) {
            return $found.FullName
        }
    }

    return $null
}

function Get-PresentMonConsoleForPayload {
    $installed = Get-InstalledPresentMonConsolePath
    if ($installed) {
        Write-Step "Usando PresentMon Console detectado: $installed"
        return $installed
    }

    $assetName = "PresentMon-$PresentMonVersion-x64.exe"
    $cached = Join-Path $PresentMonCacheDir $assetName
    if (-not (Test-Path $cached)) {
        New-Item -ItemType Directory -Path $PresentMonCacheDir -Force | Out-Null
        $uri = "https://github.com/GameTechDev/PresentMon/releases/download/v$PresentMonVersion/$assetName"
        Write-Step "Descargando PresentMon Console $PresentMonVersion"
        Invoke-WebRequest -Uri $uri -OutFile $cached -UseBasicParsing
    }

    return $cached
}

function Write-ThirdPartyNotices {
    param([string]$Target)

    @"
Chaos Game Mode includes Intel PresentMon Console.

PresentMon
Copyright (C) 2017-2024 Intel Corporation
Source: https://github.com/GameTechDev/PresentMon
License: MIT

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"@ | Set-Content -Path $Target -Encoding UTF8
}

function Clear-DirectoryInside {
    param(
        [string]$Target,
        [string]$Parent
    )

    $targetFull = [System.IO.Path]::GetFullPath($Target)
    $parentFull = [System.IO.Path]::GetFullPath($Parent)
    if (-not $targetFull.StartsWith($parentFull, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to clear '$targetFull' because it is outside '$parentFull'"
    }

    if (Test-Path $targetFull) {
        Remove-Item -LiteralPath $targetFull -Recurse -Force
    }
    New-Item -ItemType Directory -Path $targetFull -Force | Out-Null
}

if (-not (Test-Path $CargoProject)) {
    throw "No se encontro el proyecto Rust en: $CargoProject"
}
if (-not (Test-Path $Wxs)) {
    throw "No se encontro la plantilla WiX en: $Wxs"
}

if (-not $SkipBuild) {
    Write-Step "Compilando TUI Rust en modo release"
    Push-Location $CargoProject
    try {
        cargo build --release
    } finally {
        Pop-Location
    }
}

if (-not (Test-Path $ReleaseExe)) {
    throw "No se encontro el binario release: $ReleaseExe"
}

$version = if ([string]::IsNullOrWhiteSpace($PackageVersion)) {
    Get-CargoVersion
} else {
    $PackageVersion
}
if ($version -notmatch '^\d+\.\d+\.\d+(\.\d+)?$') {
    throw "PackageVersion debe ser numerico para MSI, por ejemplo 1.0.0. Valor recibido: $version"
}
$OutputDir = [System.IO.Path]::GetFullPath($OutputDir)
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
Clear-DirectoryInside -Target $PayloadDir -Parent $OutputDir

Write-Step "Preparando payload MSI"
Copy-Item -Path $ReleaseExe -Destination (Join-Path $PayloadDir "chaosgamemode.exe") -Force
foreach ($file in @("config.toml", "theme.toml")) {
    $source = Join-Path $CargoProject $file
    if (Test-Path $source) {
        Copy-Item -Path $source -Destination (Join-Path $PayloadDir $file) -Force
    }
}
Copy-Item -Path (Join-Path $ProjectRoot "README.md") -Destination (Join-Path $PayloadDir "README.md") -Force

$presentMonSource = Get-PresentMonConsoleForPayload
Copy-Item -Path $presentMonSource -Destination (Join-Path $PayloadDir "presentmon.exe") -Force
Write-ThirdPartyNotices -Target (Join-Path $PayloadDir "THIRD_PARTY_NOTICES.txt")

$wix = Get-Wix3Toolset
$wix4 = if (-not $wix) { Get-Wix4Toolset } else { $null }
if (-not $wix -and -not $wix4 -and $InstallWix) {
    Install-Wix4Toolset
    $wix4 = Get-Wix4Toolset
}

$wixobj = Join-Path $OutputDir "chaosgamemode.wixobj"
$msi = Join-Path $OutputDir "ChaosGameMode-$version-x64.msi"

if ($wix) {
    Write-Step "Compilando WiX v3"
    & $wix.Candle -nologo -arch x64 "-dPayloadDir=$PayloadDir" "-dVersion=$version" -out $wixobj $Wxs
    & $wix.Light -nologo -out $msi $wixobj
} else {
    if (-not $wix4) {
        throw "WiX Toolset no esta instalado. Ejecuta: .\build-msi.ps1 -InstallWix"
    }

    Write-Step "Compilando WiX v4"
    $wix4Source = Join-Path $OutputDir "chaosgamemode.v4.wxs"
    Copy-Item -Path $Wxs -Destination $wix4Source -Force
    Invoke-Wix4 -WixExe $wix4 -Arguments @("convert", $wix4Source)
    Invoke-Wix4 -WixExe $wix4 -Arguments @(
        "build",
        $wix4Source,
        "-arch",
        "x64",
        "-d",
        "PayloadDir=$PayloadDir",
        "-d",
        "Version=$version",
        "-out",
        $msi
    )
}

Write-Host ""
Write-Host "MSI generado: $msi" -ForegroundColor Green
Write-Host "Instalar:     msiexec /i `"$msi`""
Write-Host "Desinstalar:  desde Configuracion > Apps o msiexec /x `"$msi`""
