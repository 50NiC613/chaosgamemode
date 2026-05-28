# Chaos Game Mode

Native Windows TUI for a gaming-focused performance workflow. It scans what is running, highlights removable background load, manages a Steam-first game session, and can apply a reversible Overdrive profile before launching a game.

The app is written in Rust with Ratatui. It is designed for a second monitor: dense telemetry, keyboard-first navigation, fast startup, and terminal themes inspired by cyberpunk, Gruvbox, Tokyo Night, Catppuccin, and other power-user palettes.

> Current scope: Windows 10/11 + Steam. Epic Games, GOG, Xbox/PC Game Pass, Battle.net, and manual game folders are roadmap items.

## Table of Contents

- [English](#english)
- [Español](#espanol)
- [Development](#development)
- [GitHub Releases](#github-releases)
- [Contributing](docs/onboarding.md)

## English

### What It Does

Chaos Game Mode is a terminal dashboard and optimizer for Windows gaming sessions:

- Shows live CPU, RAM, GPU, VRAM, temperature, FPS, 1% low, frametime, and removable background memory where data is available.
- Detects Steam libraries, lists installed games, launches games normally or with an Overdrive preview flow.
- Tracks active Steam sessions and shows elapsed game time.
- Opens a dedicated `Frames` tab for MangoHUD-style FPS, frametime, RTSS status, GPU/CPU traces, and active game context.
- Shows a Frames hook log so RTSS, Steam session, game process detection, FPS capture, and overlay state can be checked at a glance.
- Can publish a lightweight fullscreen OSD through RivaTuner Statistics Server (RTSS), so FPS/session metrics can appear over exclusive fullscreen games.
- Uses configurable RTSS HUD presets (`Compact`, `MangoHUD`, `Debug`) with toggles for FPS detail, GPU/CPU/RAM, waste, session, profile, and target process.
- Warns when an Overdrive session collapses under very low FPS for several seconds, without restoring automatically.
- Marks processes as `TARGET`, `KEEP`, `WATCH`, or `HIDDEN` so you can tune what the optimizer should touch.
- Protects important apps by default, including SteelSeries tools.
- Applies reversible Overdrive actions: high-performance power plan, service cleanup, Steam priority, process cleanup, and optional Explorer handling by profile.
- Restores the system after a session by restarting shell/services and returning the power profile.
- Persists process policy, telemetry intervals, overlay settings, and UI language in `config.toml`.
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

FPS, frametime, average FPS, and 1% low are read from RivaTuner Statistics Server (RTSS) shared memory. Chaos Game Mode does not bundle or launch a separate FPS capture executable.

Install RTSS separately and keep it running while you play. The usual path is installing RivaTuner Statistics Server directly or through MSI Afterburner, then enabling the RTSS On-Screen Display for the game.

If RTSS is missing or closed, the app still runs. CPU/RAM/GPU/session data stays available, while FPS, frametime, and 1% low remain unavailable until RTSS exposes hooked game frames.

Chaos Game Mode automatically waits for Steam launchers and helper windows, then locks onto the active RTSS game process once real frame samples appear.

When a Steam game is launched or auto-detected, the TUI can move into the `Frames` tab so the second monitor becomes a focused performance view instead of crowding the main dashboard.

The `Frames` tab includes a hook status panel, short hook log, and `RTSS CANDIDATES` diagnostics. Use it to verify RTSS readiness, the active Steam session, the resolved game process, live FPS samples, rejected launcher/reporter candidates, and the fullscreen overlay state.

During an Overdrive-launched session, Chaos Game Mode also watches for severe FPS collapse. If live FPS stays below 10 after the warmup window, the Frames tab and RTSS overlay show a guard warning so you can press `2` to restore the system. The `Performance Guard` panel keeps the last Overdrive profile, impact, power/shell/service scope, action context, and profiler suggestions visible while you diagnose the drop.

### Fullscreen Overlay

Exclusive fullscreen overlays need a low-level OSD host. Chaos Game Mode uses an RTSS shared-memory backend for this instead of a normal transparent desktop window.

Requirements for the overlay:

- RivaTuner Statistics Server running.
- RTSS On-Screen Display enabled for the game.
- A hooked game process visible in RTSS shared memory.

Controls and config:

- Press `Shift+F12` to toggle the overlay while Chaos Game Mode is running, even when the game is focused. `O` also works from the `Frames` or `Settings` tab.
- The hotkey uses a global Windows keyboard hook first and falls back to the terminal/local hotkey when the hook is unavailable.
- The overlay only renders while a Steam session is active; when no game is detected it stays armed and keeps the OSD clean.
- The `Settings` tab can tune the HUD without editing files: `H` cycles layout preset, `D` toggles FPS details, and `G/C/A/W/S/P/T` toggle GPU, CPU, RAM, waste, session, profile, and target.
- HUD changes are saved for the active Steam game when a session is running; otherwise they update the global default.
- The `Steam` tab can also tune the selected game's HUD before launch: `H` cycles the preset, `F/G/C/A/W/Y/R/T` toggle fields, and `X` resets that game back to the global default.
- RTSS controls final OSD placement, font, scale, and color styling. Chaos Game Mode controls the text payload and update cadence.

```toml
[overlay]
enabled = true
backend = "rtss"
update_ms = 100
layout = "mangohud"       # compact, mangohud, debug
show_frame_stats = true   # avg, 1% low, frametime
show_gpu = true
show_cpu = true
show_ram = true
show_waste = true
show_session = true
show_profile = true
show_target = false

[overlay_game_profiles."1091500"] # per-game override, Cyberpunk 2077 example
layout = "mangohud"
show_frame_stats = true
show_gpu = true
show_cpu = true
show_ram = true
show_waste = true
show_session = true
show_profile = true
show_target = false
```

### Installation

Requirements:

- Windows 10 or Windows 11
- PowerShell 7 recommended
- Rust stable with the MSVC toolchain
- RivaTuner Statistics Server for FPS metrics and fullscreen OSD
- Internet access only if `build-msi.ps1 -InstallWix` needs to install WiX locally

Install or update from the repository:

```powershell
cd C:\src\chaosgamemode
.\install.ps1
```

Run from any new terminal:

```powershell
chaosgamemode
```

Run a quick installation/RTSS diagnostic:

```powershell
chaosgamemode --doctor
```

Dump the live RTSS frame entries while a game is open:

```powershell
chaosgamemode --rtss-dump
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
- `config.default.toml`
- `theme.default.toml`
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
config_version = 4

[ui]
language = "es" # es or en

[telemetry]
telemetry_ms = 1000
process_ms = 3000
platform_ms = 15000

[overlay]
enabled = true
backend = "rtss"
update_ms = 100
layout = "mangohud"
show_frame_stats = true
show_gpu = true
show_cpu = true
show_ram = true
show_waste = true
show_session = true
show_profile = true
show_target = false

[overlay_game_profiles."1091500"] # Cyberpunk 2077 example
layout = "mangohud"
show_frame_stats = true
show_gpu = true
show_cpu = true
show_ram = true
show_waste = true
show_session = true
show_profile = true
show_target = false
```

Profiles:

- `safe`: conservative cleanup, keeps Explorer alive, no service stop list by default.
- `balanced`: recommended daily profile; closes known background apps but keeps Windows services running by default.
- `aggressive`: strongest cleanup, can stop configured services and restart Explorer when configured.

Press `3` inside the TUI to cycle `safe -> balanced -> aggressive` without editing `config.toml`.

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

Navigation, header, footer, dialogs, theme menu, and settings chrome are language-aware. Some deep telemetry/status strings still come from Windows, Steam, RTSS, or the internal action log and may remain mixed while the app evolves.

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
- GPU/OSD/control tools such as AMD/Radeon, RTSS, RivaTuner, MSI Afterburner, and SteelSeries are protected internally even if an older config accidentally lists them as cleanup targets.
- Windows system processes and Defender-related entries are hidden from normal process targeting.
- Existing old configs are migrated so the default `balanced` profile no longer stops services; custom service lists are kept.
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
- Abre una tab dedicada `Frames`, estilo MangoHUD, con FPS, frametime, estado de RTSS, trazas GPU/CPU y contexto del juego activo.
- Muestra un hook log en `Frames` para revisar RTSS, sesion de Steam, deteccion del proceso del juego, captura FPS y estado del overlay.
- Puede publicar un OSD ligero via RivaTuner Statistics Server (RTSS), para ver metricas encima de juegos en fullscreen exclusivo.
- Usa presets configurables de HUD en RTSS (`Compact`, `MangoHUD`, `Debug`) con toggles para detalle FPS, GPU/CPU/RAM, waste, sesion, perfil y proceso target.
- Avisa si una sesion con Overdrive cae por debajo de FPS muy bajos durante varios segundos, sin restaurar automaticamente.
- Clasifica procesos como `TARGET`, `KEEP`, `WATCH` u `HIDDEN` para decidir que se puede cerrar y que debe respetarse.
- Protege apps importantes por defecto, incluyendo herramientas de SteelSeries.
- Aplica acciones reversibles: plan de energia de alto rendimiento, limpieza de servicios, prioridad a Steam, limpieza de procesos y manejo opcional de Explorer segun perfil.
- Restaura el sistema despues de jugar.
- Guarda politica de procesos, intervalos de telemetria, overlay e idioma en `config.toml`.
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

### FPS Y Frames

Los FPS, frametime, FPS promedio y 1% low se leen desde la memoria compartida de RivaTuner Statistics Server (RTSS). Chaos Game Mode no incluye ni ejecuta un capturador externo de FPS dentro del MSI.

Instala RTSS por separado y dejalo abierto mientras juegas. Lo normal es instalar RivaTuner Statistics Server directamente o mediante MSI Afterburner, y activar el On-Screen Display de RTSS para el juego.

Si RTSS no esta instalado o esta cerrado, la app sigue funcionando. CPU/RAM/GPU/sesion siguen disponibles; FPS, frametime y 1% low quedan como no disponibles hasta que RTSS exponga frames reales del juego.

Chaos Game Mode espera automaticamente launchers y ventanas auxiliares de Steam, y se queda con el proceso activo cuando RTSS empieza a entregar samples de frames reales.

La tab `Frames` incluye un panel de hook status, un hook log corto y diagnostico `RTSS CANDIDATES`. Sirve para confirmar si RTSS esta listo, si hay sesion Steam activa, que proceso del juego se resolvio, si ya hay FPS en vivo, que launchers/reporters se rechazaron y si el overlay fullscreen esta activo.

Durante una sesion lanzada con Overdrive, Chaos Game Mode tambien vigila caidas severas de FPS. Si los FPS se quedan por debajo de 10 despues de la ventana inicial, la tab Frames y el overlay RTSS muestran una alerta para que puedas pulsar `2` y restaurar el sistema. El panel `Performance Guard` mantiene visible el ultimo perfil Overdrive, impacto, alcance de power/shell/servicios, contexto de acciones y sugerencias del profiler mientras diagnosticas la caida.

### Instalacion

Requisitos:

- Windows 10 o Windows 11
- PowerShell 7 recomendado
- Rust estable con toolchain MSVC
- RivaTuner Statistics Server para metricas FPS y OSD en fullscreen
- Acceso a internet solo si `build-msi.ps1 -InstallWix` necesita instalar WiX localmente

Instalar o actualizar desde el repo:

```powershell
cd C:\src\chaosgamemode
.\install.ps1
```

Ejecutar desde una terminal nueva:

```powershell
chaosgamemode
```

Diagnostico rapido de instalacion/RTSS:

```powershell
chaosgamemode --doctor
```

Volcar las entradas FPS que RTSS esta exponiendo mientras el juego esta abierto:

```powershell
chaosgamemode --rtss-dump
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

La navegacion, header, footer, modales, menu de temas y chrome de ajustes ya usan este ajuste. Algunos textos profundos de telemetria, Steam, RTSS o logs internos pueden seguir mixtos mientras se termina la cobertura completa.

### Configuracion

Archivo principal:

```text
tui-rs\config.toml
```

Secciones clave:

- `[ui]`: idioma de la interfaz.
- `[telemetry]`: frecuencia de CPU/RAM/procesos/plataforma.
- `[overlay]`: backend RTSS, activacion del OSD, frecuencia de actualizacion y campos visibles del HUD.
- `[profiles.safe]`, `[profiles.balanced]`, `[profiles.aggressive]`: perfiles de optimizacion.

Perfiles:

- `safe`: limpieza conservadora, no detiene servicios.
- `balanced`: recomendado para uso diario; cierra apps de fondo conocidas, pero ya no detiene servicios por defecto.
- `aggressive`: limpieza fuerte; puede detener servicios configurados y reiniciar Explorer si se activa.

Pulsa `3` dentro de la TUI para alternar `safe -> balanced -> aggressive` sin editar `config.toml`.

Politica de procesos:

- `processes`: procesos candidatos para cerrar.
- `protected_processes`: procesos protegidos que no se cierran.
- `hidden_processes`: procesos que no deben ensuciar la lista principal.

### Overlay Fullscreen

Los overlays encima de fullscreen exclusivo necesitan un host OSD de bajo nivel. Chaos Game Mode usa RTSS por memoria compartida para evitar una ventana transparente normal, que no es confiable en fullscreen exclusivo.

Requisitos:

- RivaTuner Statistics Server abierto.
- OSD de RTSS activado para el juego.
- Un proceso de juego hookeado y visible en RTSS shared memory.

Uso:

- Pulsa `Shift+F12` para activar/desactivar el overlay mientras Chaos Game Mode esta abierto, incluso con el juego enfocado. `O` tambien funciona desde `Frames` o `Ajustes`.
- El overlay solo se dibuja cuando hay una sesion de Steam activa; sin juego abierto queda armado y no ensucia la pantalla.
- La tab `Ajustes` permite tunear el HUD sin editar archivos: `H` cambia el preset, `D` activa/desactiva detalles FPS, y `G/C/A/W/S/P/T` alternan GPU, CPU, RAM, waste, sesion, perfil y target.
- Si hay una sesion Steam activa, los cambios del HUD se guardan para ese juego; si no hay juego activo, actualizan el default global.
- La tab `Steam` tambien permite tunear el HUD del juego seleccionado antes de lanzarlo: `H` cambia el preset, `F/G/C/A/W/Y/R/T` alternan campos y `X` devuelve ese juego al default global.
- RTSS controla la posicion final, fuente, escala y estilo de color del OSD. Chaos Game Mode controla el texto y la frecuencia de actualizacion.

```toml
[overlay]
enabled = true
backend = "rtss"
update_ms = 100
layout = "mangohud"       # compact, mangohud, debug
show_frame_stats = true   # avg, 1% low, frametime
show_gpu = true
show_cpu = true
show_ram = true
show_waste = true
show_session = true
show_profile = true
show_target = false

[overlay_game_profiles."1091500"] # override por juego, ejemplo Cyberpunk 2077
layout = "mangohud"
show_frame_stats = true
show_gpu = true
show_cpu = true
show_ram = true
show_waste = true
show_session = true
show_profile = true
show_target = false
```

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
cd C:\src\chaosgamemode\tui-rs
cargo run
```

Recommended live development loop:

```powershell
cd C:\src\chaosgamemode
.\dev-watch.ps1
```

Quality checks:

```powershell
cd C:\src\chaosgamemode\tui-rs
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
tui-rs\src\config.rs        profiles, process policy, overlay, UI config
tui-rs\src\steam.rs         Steam library discovery and URI commands
tui-rs\src\frames.rs        RTSS frame probing and FPS capture
tui-rs\src\overlay.rs       RTSS fullscreen overlay backend
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

The MSI build packages Chaos Game Mode, default config/theme files, fallback `*.default.toml` templates, and the README. RTSS is intentionally documented as an external requirement instead of being bundled inside the installer.

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
- RivaTuner Statistics Server for FPS metrics and fullscreen OSD
```

## License

Released under the [MIT License](LICENSE).
