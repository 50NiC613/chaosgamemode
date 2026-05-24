# Chaos Game Mode

Native Windows TUI for a gaming-focused performance workflow. It scans what is running, highlights removable background load, manages a Steam-first game session, and can apply a reversible Overdrive profile before launching a game.

The app is written in Rust with Ratatui. It is designed for a second monitor: dense telemetry, keyboard-first navigation, fast startup, and terminal themes inspired by cyberpunk, Gruvbox, Tokyo Night, Catppuccin, and other power-user palettes.

> Current scope: Windows 10/11 + Steam. Epic Games, GOG, Xbox/PC Game Pass, Battle.net, and manual game folders are roadmap items.

## Table of Contents

- [English](#english)
- [Español](#espanol)
- [Development](#development)
- [GitHub Releases](#github-releases)

## English

### What It Does

Chaos Game Mode is a terminal dashboard and optimizer for Windows gaming sessions:

- Shows live CPU, RAM, GPU, VRAM, temperature, FPS, 1% low, frametime, and removable background memory where data is available.
- Detects Steam libraries, lists installed games, launches games normally or with an Overdrive preview flow.
- Tracks active Steam sessions and shows elapsed game time.
- Opens a dedicated `Frames` tab for MangoHUD-style FPS, frametime, PresentMon status, GPU/CPU traces, and active game context.
- Marks processes as `TARGET`, `KEEP`, `WATCH`, or `HIDDEN` so you can tune what the optimizer should touch.
- Protects important apps by default, including SteelSeries tools.
- Applies reversible Overdrive actions: high-performance power plan, service cleanup, Steam priority, process cleanup, and optional Explorer handling by profile.
- Restores the system after a session by restarting shell/services and returning the power profile.
- Persists process policy, telemetry intervals, integrations, and UI language in `config.toml`.
- Supports live theme switching through `theme.toml`.

### Steam Support

Steam is the first-class platform today.

Supported Steam features:

- Local library discovery through Steam library folders and `appmanifest_*.acf` files.
- Game list navigation from the TUI.
- Normal launch with `steam://run/<appid>`.
- Overdrive launch with a confirmation preview before changes are applied.
- Install, validate files, open properties, open downloads, and uninstall through official Steam URI commands.
- Automatic session timer when a known Steam game is running.

The app does not ask for your Steam password and does not need Steam login tokens. It uses local Steam metadata and Steam URI commands.

### FPS And Frame Data

FPS metrics use Intel PresentMon Console. The MSI bundles `presentmon.exe` beside `chaosgamemode.exe`, so normal installs do not need a separate WinGet step. For portable/dev runs, you can still install it manually:

```powershell
winget install Intel.PresentMon.Console
```

Chaos Game Mode will look for PresentMon in this order:

1. `presentmon_exe` in `config.toml`
2. `PRESENTMON_EXE` environment variable
3. bundled `presentmon.exe` next to `chaosgamemode.exe`
4. the default WinGet installation path
5. `PATH`

If PresentMon is missing, the app still runs. FPS, frametime, and 1% low will show as unavailable until the tool is detected.

When a Steam game is launched or auto-detected, the TUI can move into the `Frames` tab so the second monitor becomes a focused performance view instead of crowding the main dashboard.

### Installation

Requirements:

- Windows 10 or Windows 11
- PowerShell 7 recommended
- Rust stable with the MSVC toolchain
- Internet access when building the MSI, so `build-msi.ps1` can download the bundled PresentMon binary

Install or update from the repository:

```powershell
cd D:\Dev\chaosgamemode
.\install.ps1
```

Run from any new terminal:

```powershell
chaosgamemode
```

Update the installed app after code changes:

```powershell
chaosgamemode-update
```

Or from the repository:

```powershell
.\install.ps1 -Action Update
```

Uninstall:

```powershell
chaosgamemode-uninstall
```

Or:

```powershell
.\install.ps1 -Action Uninstall
```

Git Bash, MSYS, and Cygwin users can call the wrapper:

```bash
./install.sh install
./install.sh update
./install.sh uninstall
```

### MSI Installer

The repository includes a WiX v3 MSI template for per-user installation.

```powershell
.\build-msi.ps1 -InstallWix
```

Expected output:

```text
dist\ChaosGameMode-<version>-x64.msi
```

The MSI installs:

- `chaosgamemode.exe`
- `config.toml`
- `theme.toml`
- `README.md`
- Start-menu shortcuts
- user `PATH` entry for `%LOCALAPPDATA%\Programs\ChaosGameMode`

### Configuration

Main configuration lives in:

```text
tui-rs\config.toml
```

Important sections:

```toml
active_profile = "balanced"

[ui]
language = "es" # es or en

[telemetry]
telemetry_ms = 1000
process_ms = 3000
platform_ms = 15000

[integrations]
# presentmon_exe = "D:\\Tools\\PresentMon.exe"
```

Profiles:

- `safe`: conservative cleanup, keeps Explorer alive, no service stop list by default.
- `balanced`: recommended daily profile.
- `aggressive`: strongest cleanup, can restart Explorer when configured.

Process policy:

- `processes`: candidates for cleanup.
- `protected_processes`: never close these from Overdrive.
- `hidden_processes`: hide Windows/system noise from the process tab.

### Languages

The TUI has a configurable language foundation:

```toml
[ui]
language = "es"
```

Supported values:

- `es`: Spanish
- `en`: English

Navigation, header, footer, dialogs, theme menu, and settings chrome are language-aware. Some deep telemetry/status strings still come from Windows, Steam, PresentMon, or the internal action log and may remain mixed while the app evolves.

### Themes

Runtime theme configuration lives in:

```text
tui-rs\theme.toml
```

Available presets include:

- Cyberpunk
- Hacker
- Gruvbox
- Tokyo Night
- Catppuccin Mocha
- Dracula
- Nord
- Rosé Pine

Press `M` inside the TUI to cycle/apply theme presets.

### Safety Model

Chaos Game Mode is intentionally local and reversible:

- Overdrive requires a preview/confirmation step.
- Steam uninstall is delegated to Steam through `steam://uninstall/<appid>`.
- Protected processes are excluded from cleanup.
- Windows system processes and Defender-related entries are hidden from normal process targeting.
- Restore brings back shell/services and balanced power behavior.

### Roadmap

Planned platform expansion:

- Epic Games library discovery
- GOG Galaxy discovery
- Xbox/PC Game Pass detection
- Battle.net detection
- Manual game folders for non-store games
- Per-game profiles
- Export/import profile presets
- More complete bilingual copy coverage across every panel

## Espanol

### Que Hace

Chaos Game Mode es un dashboard y optimizador en terminal para sesiones de gaming en Windows:

- Muestra CPU, RAM, GPU, VRAM, temperaturas, FPS, 1% low, frametime y memoria recuperable cuando esos datos estan disponibles.
- Detecta bibliotecas de Steam, lista juegos instalados y permite lanzarlos normal o con Overdrive.
- Lleva un contador de sesion cuando un juego de Steam esta activo.
- Abre una tab dedicada `Frames`, estilo MangoHUD, con FPS, frametime, estado de PresentMon, trazas GPU/CPU y contexto del juego activo.
- Clasifica procesos como `TARGET`, `KEEP`, `WATCH` u `HIDDEN` para decidir que se puede cerrar y que debe respetarse.
- Protege apps importantes por defecto, incluyendo herramientas de SteelSeries.
- Aplica acciones reversibles: plan de energia de alto rendimiento, limpieza de servicios, prioridad a Steam, limpieza de procesos y manejo opcional de Explorer segun perfil.
- Restaura el sistema despues de jugar.
- Guarda politica de procesos, intervalos de telemetria, integraciones e idioma en `config.toml`.
- Permite cambiar temas desde `theme.toml`.

### Soporte De Steam

Steam es la plataforma principal en este momento.

Funciones actuales:

- Deteccion local de bibliotecas de Steam y archivos `appmanifest_*.acf`.
- Navegacion de juegos desde la TUI.
- Lanzamiento normal con `steam://run/<appid>`.
- Lanzamiento con Overdrive y pantalla de confirmacion.
- Instalar, validar archivos, abrir propiedades, abrir descargas y desinstalar usando comandos URI oficiales de Steam.
- Temporizador automatico cuando se detecta un juego conocido de Steam.

La app no pide tu contraseña de Steam ni usa tokens de login. Trabaja con metadatos locales y comandos URI de Steam.

### Instalacion

Requisitos:

- Windows 10 o Windows 11
- PowerShell 7 recomendado
- Rust estable con toolchain MSVC
- Acceso a internet al construir el MSI, para que `build-msi.ps1` descargue el binario incluido de PresentMon

Instalar o actualizar desde el repo:

```powershell
cd D:\Dev\chaosgamemode
.\install.ps1
```

Ejecutar desde una terminal nueva:

```powershell
chaosgamemode
```

Actualizar despues de cambiar el codigo:

```powershell
chaosgamemode-update
```

O desde el repo:

```powershell
.\install.ps1 -Action Update
```

Desinstalar:

```powershell
chaosgamemode-uninstall
```

### Idioma

El idioma se configura en:

```toml
[ui]
language = "es"
```

Valores soportados:

- `es`: español
- `en`: ingles

La navegacion, header, footer, modales, menu de temas y chrome de ajustes ya usan este ajuste. Algunos textos profundos de telemetria, Steam, PresentMon o logs internos pueden seguir mixtos mientras se termina la cobertura completa.

### Configuracion

Archivo principal:

```text
tui-rs\config.toml
```

Secciones clave:

- `[ui]`: idioma de la interfaz.
- `[telemetry]`: frecuencia de CPU/RAM/procesos/plataforma.
- `[integrations]`: ruta opcional a PresentMon.
- `[profiles.safe]`, `[profiles.balanced]`, `[profiles.aggressive]`: perfiles de optimizacion.

Politica de procesos:

- `processes`: procesos candidatos para cerrar.
- `protected_processes`: procesos protegidos que no se cierran.
- `hidden_processes`: procesos que no deben ensuciar la lista principal.

### PresentMon

Para FPS, frametime y 1% low se usa Intel PresentMon Console. El MSI incluye `presentmon.exe` junto a `chaosgamemode.exe`, asi que una instalacion normal no necesita un paso extra con WinGet. Para uso portable/dev, puedes instalarlo manualmente con:

```powershell
winget install Intel.PresentMon.Console
```

Tambien puedes fijar la ruta manualmente:

```toml
[integrations]
presentmon_exe = "D:\\Tools\\PresentMon.exe"
```

Si PresentMon no esta disponible, la TUI sigue funcionando; solo los datos de frames quedan en estado no disponible.

Cuando se lanza o detecta un juego de Steam, la TUI puede enfocarse en la tab `Frames` para que el segundo monitor sea una vista de rendimiento limpia y no sobrecargue el dashboard.

### Roadmap

Steam queda como base actual. Para futuro se plantea:

- Epic Games
- GOG Galaxy
- Xbox/PC Game Pass
- Battle.net
- carpetas manuales para juegos fuera de tiendas
- perfiles por juego
- import/export de configuraciones
- traduccion completa de todos los paneles

## Development

Run locally:

```powershell
cd D:\Dev\chaosgamemode\tui-rs
cargo run
```

Recommended live development loop:

```powershell
cd D:\Dev\chaosgamemode
.\dev-watch.ps1
```

Quality checks:

```powershell
cd D:\Dev\chaosgamemode\tui-rs
cargo fmt -- --check
cargo test
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo build --release
```

Project layout:

```text
tui-rs\src\app.rs           event loop, app state, key handling
tui-rs\src\ui.rs            main frame, header, tabs, footer, modals
tui-rs\src\ui\dashboard.rs  telemetry dashboard
tui-rs\src\ui\steam_panel.rs Steam library and session panels
tui-rs\src\ui\pages.rs      frames, processes, overdrive, system, history, settings
tui-rs\src\config.rs        profiles, process policy, integrations, UI config
tui-rs\src\steam.rs         Steam library discovery and URI commands
tui-rs\src\presentmon.rs    PresentMon probing and frame capture
tui-rs\src\system.rs        Windows state and Overdrive/Restore actions
tui-rs\src\theme.rs         theme presets and live theme file
tui-rs\src\i18n.rs          UI language strings
```

## GitHub Releases

Releases are automated by `.github\workflows\release.yml`.

Recommended flow:

1. Update `tui-rs\Cargo.toml` version.
2. Run the quality checks from the development section.
3. Create a test tag first:

```powershell
git tag -a v0.1.0-test -m "Chaos Game Mode v0.1.0-test"
git push origin v0.1.0-test
```

Tags ending in `-test` create prereleases, which are useful for validating the MSI workflow before publishing a real version.

4. Create the real release tag:

```powershell
git tag -a v1.0.0 -m "Chaos Game Mode v1.0.0"
git push origin v1.0.0
```

The workflow builds and uploads:

- `ChaosGameMode-v<version>-x64.msi`
- `chaosgamemode.exe`
- `config.toml`
- `theme.toml`
- generated release notes

Local MSI builds are still supported:

```powershell
.\build-msi.ps1 -InstallWix
```

The MSI build copies `presentmon.exe` into the installer payload. If PresentMon is not already installed locally, the build script downloads the configured PresentMon release into `.tools\presentmon`.

Suggested manual release notes structure:

```markdown
## Added
- ...

## Changed
- ...

## Fixed
- ...

## Install
Run the MSI or use .\install.ps1 from the repository.

## Requirements
- Windows 10/11
- Optional: Intel PresentMon Console for FPS metrics
```

## License

No license has been declared yet. Add one before accepting external contributions or publishing official releases.
