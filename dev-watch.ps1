param(
    [switch]$Release
)

$ErrorActionPreference = "Stop"

$ProjectDir = Join-Path $PSScriptRoot "tui-rs"
$CargoArgs = @("run")

if ($Release) {
    $CargoArgs += "--release"
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "Rust/Cargo no esta disponible en PATH. Instala Rust desde https://rustup.rs"
    exit 1
}

Push-Location $ProjectDir
try {
    if (Get-Command cargo-watch -ErrorAction SilentlyContinue) {
        $WatchArgs = @("watch", "-w", "src", "-w", "Cargo.toml", "-x", ($CargoArgs -join " "))
        & cargo @WatchArgs
        exit $LASTEXITCODE
    }

    Write-Warning "cargo-watch no esta instalado; ejecuto la app una sola vez."
    Write-Host "Para autoreinicio al guardar: cargo install cargo-watch --locked" -ForegroundColor Cyan
    & cargo @CargoArgs
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
