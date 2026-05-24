[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [ValidateSet("Install", "Update", "Uninstall")]
    [string]$Action = "Install",

    [Parameter(Mandatory = $false)]
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA "Programs\ChaosGameMode"),

    [Parameter(Mandatory = $false)]
    [switch]$NoPath,

    [Parameter(Mandatory = $false)]
    [switch]$SkipPresentMon
)

$ErrorActionPreference = "Stop"

$ProjectRoot = $PSScriptRoot
$CargoProject = Join-Path $ProjectRoot "tui-rs"
$BinaryName = "chaosgamemode.exe"
$ReleaseExe = Join-Path $CargoProject "target\release\$BinaryName"
$ThemeSource = Join-Path $CargoProject "theme.toml"
$ConfigSource = Join-Path $CargoProject "config.toml"
$InstallDir = [System.IO.Path]::GetFullPath($InstallDir)
$InstalledExe = Join-Path $InstallDir $BinaryName
$InstalledTheme = Join-Path $InstallDir "theme.toml"
$InstalledConfig = Join-Path $InstallDir "config.toml"
$InstallerPath = Join-Path $ProjectRoot "install.ps1"
$UpdateCmd = Join-Path $InstallDir "chaosgamemode-update.cmd"
$UninstallCmd = Join-Path $InstallDir "chaosgamemode-uninstall.cmd"

function Write-Step {
    param([string]$Message)
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Normalize-PathEntry {
    param([string]$Value)

    if ([string]::IsNullOrWhiteSpace($Value)) {
        return ""
    }

    try {
        return ([System.IO.Path]::GetFullPath($Value)).TrimEnd("\").ToLowerInvariant()
    } catch {
        return $Value.TrimEnd("\").ToLowerInvariant()
    }
}

function Get-PathEntries {
    param([string]$Value)

    if ([string]::IsNullOrWhiteSpace($Value)) {
        return @()
    }

    return @($Value -split ";" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
}

function Test-PathContains {
    param(
        [string[]]$Entries,
        [string]$Directory
    )

    $target = Normalize-PathEntry $Directory
    foreach ($entry in $Entries) {
        if ((Normalize-PathEntry $entry) -eq $target) {
            return $true
        }
    }

    return $false
}

function Add-ToUserPath {
    param([string]$Directory)

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $userEntries = Get-PathEntries $userPath
    if (-not (Test-PathContains $userEntries $Directory)) {
        $newUserPath = if ([string]::IsNullOrWhiteSpace($userPath)) {
            $Directory
        } else {
            "$userPath;$Directory"
        }
        [Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
        Write-Step "Agregado al PATH de usuario: $Directory"
    } else {
        Write-Step "PATH de usuario ya contiene: $Directory"
    }

    $processEntries = Get-PathEntries $env:Path
    if (-not (Test-PathContains $processEntries $Directory)) {
        $env:Path = if ([string]::IsNullOrWhiteSpace($env:Path)) {
            $Directory
        } else {
            "$env:Path;$Directory"
        }
    }
}

function Remove-FromUserPath {
    param([string]$Directory)

    $target = Normalize-PathEntry $Directory

    $userEntries = Get-PathEntries ([Environment]::GetEnvironmentVariable("Path", "User"))
    $newUserEntries = @($userEntries | Where-Object { (Normalize-PathEntry $_) -ne $target })
    [Environment]::SetEnvironmentVariable("Path", ($newUserEntries -join ";"), "User")

    $processEntries = Get-PathEntries $env:Path
    $newProcessEntries = @($processEntries | Where-Object { (Normalize-PathEntry $_) -ne $target })
    $env:Path = $newProcessEntries -join ";"

    Write-Step "Quitado del PATH de usuario: $Directory"
}

function Write-CommandShim {
    param(
        [string]$Path,
        [string]$ShimAction
    )

    if ($ShimAction -eq "Uninstall") {
        $psInstallerPath = $InstallerPath.Replace("'", "''")
        $psInstallDir = $InstallDir.Replace("'", "''")
        $psCommand = "Start-Sleep -Milliseconds 300; & '$psInstallerPath' -Action Uninstall -InstallDir '$psInstallDir' %*"
        $contents = @(
            "@echo off",
            "start `"`" /B powershell.exe -NoProfile -ExecutionPolicy Bypass -Command `"$psCommand`"",
            "exit /b"
        )
        Set-Content -Path $Path -Value $contents -Encoding ASCII
        return
    }

    $contents = @(
        "@echo off",
        "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$InstallerPath`" -Action $ShimAction -InstallDir `"$InstallDir`" %* & exit /b"
    )
    Set-Content -Path $Path -Value $contents -Encoding ASCII
}

function Get-PresentMonConsolePath {
    foreach ($commandName in @("presentmon", "presentmon.exe", "PresentMon.exe")) {
        $command = Get-Command $commandName -ErrorAction SilentlyContinue
        if ($command -and $command.Source -and (Test-Path $command.Source)) {
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

function Install-PresentMonConsole {
    if ($SkipPresentMon) {
        Write-Step "PresentMon omitido por -SkipPresentMon"
        return
    }

    $presentMonPath = Get-PresentMonConsolePath
    if ($presentMonPath) {
        Write-Step "PresentMon Console detectado: $presentMonPath"
        return
    }

    $winget = Get-Command winget -ErrorAction SilentlyContinue
    if (-not $winget) {
        Write-Warning "winget no esta disponible; FPS/frametime quedara desactivado hasta instalar Intel.PresentMon.Console."
        return
    }

    Write-Step "Instalando dependencia: Intel.PresentMon.Console"
    try {
        & winget install --id Intel.PresentMon.Console --exact --accept-package-agreements --accept-source-agreements
    } catch {
        Write-Warning "No se pudo instalar Intel.PresentMon.Console automaticamente: $_"
        return
    }

    $presentMonPath = Get-PresentMonConsolePath
    if ($presentMonPath) {
        Write-Step "PresentMon Console instalado: $presentMonPath"
    } else {
        Write-Warning "PresentMon fue instalado por winget, pero la terminal actual podria necesitar reiniciarse para exponer el comando presentmon."
    }
}

if ($Action -eq "Uninstall") {
    if (-not $NoPath) {
        Remove-FromUserPath $InstallDir
    }

    if (Test-Path $InstallDir) {
        Write-Step "Eliminando instalacion: $InstallDir"
        Remove-Item -LiteralPath $InstallDir -Recurse -Force
    }

    Write-Host ""
    Write-Host "Chaos Game Mode fue desinstalado." -ForegroundColor Green
    exit 0
}

if (-not (Test-Path $CargoProject)) {
    throw "No se encontro el proyecto Rust en: $CargoProject"
}

Get-Command cargo -ErrorAction Stop | Out-Null

Install-PresentMonConsole

Write-Step "Compilando TUI Rust en modo release"
Push-Location $CargoProject
try {
    cargo build --release
} finally {
    Pop-Location
}

if (-not (Test-Path $ReleaseExe)) {
    throw "No se encontro el binario compilado: $ReleaseExe"
}

Write-Step "Instalando en: $InstallDir"
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null

try {
    Copy-Item -Path $ReleaseExe -Destination $InstalledExe -Force
} catch {
    throw "No se pudo reemplazar $InstalledExe. Cierra chaosgamemode si esta abierto y vuelve a ejecutar este instalador."
}

if ((Test-Path $ThemeSource) -and -not (Test-Path $InstalledTheme)) {
    Copy-Item -Path $ThemeSource -Destination $InstalledTheme
    Write-Step "Tema instalado: $InstalledTheme"
} elseif (Test-Path $InstalledTheme) {
    Write-Step "Tema existente preservado: $InstalledTheme"
}

if ((Test-Path $ConfigSource) -and -not (Test-Path $InstalledConfig)) {
    Copy-Item -Path $ConfigSource -Destination $InstalledConfig
    Write-Step "Config instalada: $InstalledConfig"
} elseif (Test-Path $InstalledConfig) {
    Write-Step "Config existente preservada: $InstalledConfig"
}

Write-CommandShim -Path $UpdateCmd -ShimAction "Update"
Write-CommandShim -Path $UninstallCmd -ShimAction "Uninstall"

if (-not $NoPath) {
    Add-ToUserPath $InstallDir
}

Write-Host ""
Write-Host "Chaos Game Mode quedo instalado." -ForegroundColor Green
Write-Host "Ejecutar:      chaosgamemode"
Write-Host "Actualizar:   chaosgamemode-update"
Write-Host "Desinstalar:  chaosgamemode-uninstall"
Write-Host ""
Write-Host "Si la terminal actual no reconoce el comando, abre una terminal nueva."
