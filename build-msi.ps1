[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [string]$OutputDir = (Join-Path $PSScriptRoot "dist\msi"),

    [Parameter(Mandatory = $false)]
    [string]$PackageVersion,

    [Parameter(Mandatory = $false)]
    [switch]$SkipBuild,

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
$Wix4Version = "4.0.6"

function Write-Step {
    param([string]$Message)
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Get-DisplayPath {
    param([string]$Path)

    $full = [System.IO.Path]::GetFullPath($Path)
    $root = [System.IO.Path]::GetFullPath($ProjectRoot)
    if ($full.StartsWith($root, [System.StringComparison]::OrdinalIgnoreCase)) {
        return $full.Substring($root.Length).TrimStart('\', '/')
    }

    return Split-Path -Leaf $full
}

function Get-CargoVersion {
    $contents = Get-Content -Raw -Path $CargoToml
    $match = [regex]::Match($contents, '(?m)^\s*version\s*=\s*"([^"]+)"')
    if (-not $match.Success) {
        throw "No se pudo leer version desde tui-rs\Cargo.toml"
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
        [string[]]$Arguments,
        [int[]]$AllowedExitCodes = @(0)
    )

    $dotnet = Get-DotNetCommand
    if ($dotnet) {
        $dotnetRoot = Split-Path -Parent $dotnet
        $env:DOTNET_ROOT = $dotnetRoot
        $env:PATH = "$dotnetRoot;$env:PATH"
    }

    $output = & $WixExe @Arguments 2>&1
    $exitCode = $LASTEXITCODE
    $output
    if ($AllowedExitCodes -notcontains $exitCode) {
        throw "WiX command failed: wix $($Arguments[0])"
    }
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
    throw "No se encontro el proyecto Rust en tui-rs"
}
if (-not (Test-Path $Wxs)) {
    throw "No se encontro la plantilla WiX en packaging\wix\chaosgamemode.wxs"
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
    throw "No se encontro el binario release en tui-rs\target\release\chaosgamemode.exe"
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
    Push-Location $ProjectRoot
    try {
        & $wix.Candle -nologo -arch x64 "-dPayloadDir=$PayloadDir" "-dVersion=$version" -out $wixobj "packaging\wix\chaosgamemode.wxs"
        & $wix.Light -nologo -out $msi $wixobj
    } finally {
        Pop-Location
    }
} else {
    if (-not $wix4) {
        throw "WiX Toolset no esta instalado. Ejecuta: .\build-msi.ps1 -InstallWix"
    }

    Write-Step "Compilando WiX v4"
    $wix4Source = Join-Path $OutputDir "chaosgamemode.v4.wxs"
    $wix4SourceDisplay = Get-DisplayPath $wix4Source
    Copy-Item -Path $Wxs -Destination $wix4Source -Force
    Push-Location $ProjectRoot
    try {
        Invoke-Wix4 -WixExe $wix4 -Arguments @("convert", $wix4SourceDisplay) -AllowedExitCodes @(0, 6) | Where-Object { $_ -notmatch 'information WIX' }
        Invoke-Wix4 -WixExe $wix4 -Arguments @(
            "build",
            $wix4SourceDisplay,
            "-arch",
            "x64",
            "-d",
            "PayloadDir=$PayloadDir",
            "-d",
            "Version=$version",
            "-out",
            (Get-DisplayPath $msi)
        )
    } finally {
        Pop-Location
    }
}

Write-Host ""
$msiDisplay = Get-DisplayPath $msi
Write-Host "MSI generado: $msiDisplay" -ForegroundColor Green
Write-Host "Instalar:     msiexec /i `"$msiDisplay`""
Write-Host "Desinstalar:  desde Configuracion > Apps o msiexec /x `"$msiDisplay`""
