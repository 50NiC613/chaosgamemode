# Developer Onboarding — Chaos Game Mode

## Prerequisites

| Requirement | Version | Notes |
|---|---|---|
| Rust | stable (MSVC toolchain) | `rustup toolchain install stable-msvc` |
| Windows | 10 or 11 | Required — Windows APIs only |
| PowerShell | 7+ recommended | For install scripts |
| Git | any recent | |
| RivaTuner Statistics Server | latest | Optional for dev; required for FPS/overlay features |

Install Rust: https://rustup.rs — choose the MSVC ABI option during setup.

## Clone and Install

```powershell
git clone https://github.com/50NiC613/chaosgamemode.git
cd chaosgamemode
.\install.ps1
```

This compiles the release binary, copies it to `%LOCALAPPDATA%\Programs\ChaosGameMode`, and adds it to your user `PATH`. After a new terminal session:

```powershell
chaosgamemode
```

## Environment Setup

No environment variables are required. Configuration lives in local TOML files — not `.env` files.

| File | Purpose | Notes |
|---|---|---|
| `tui-rs\config.toml` | Runtime config: language, profiles, telemetry, overlay | Auto-created on first run |
| `tui-rs\theme.toml` | Active theme | Auto-created on first run |
| `tui-rs\config.default.toml` | Read-only factory defaults | Do not edit |
| `tui-rs\theme.default.toml` | Read-only factory theme defaults | Do not edit |

If either config file is absent, the app creates it from the `*.default.toml` template.

To run in Spanish: set `language = "es"` under `[ui]` in `config.toml`.

## Dev Server / Run

```powershell
# Run directly (single build + run)
cd tui-rs
cargo run

# Live reload loop (rebuild + restart on save)
cd ..
.\dev-watch.ps1
```

The TUI opens full-screen in the terminal. `Q` or `Ctrl+C` exits.

## Quality Checks

Run all of these from `tui-rs\`:

```powershell
cd tui-rs

cargo fmt -- --check          # style check (no autofix)
cargo fmt                     # autofix style
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test
cargo build --release
```

CI runs these same commands in `.github\workflows\release.yml`.

## Project Structure

```
chaosgamemode\
  tui-rs\                   Rust crate — the entire application
    src\
      app.rs                Event loop, App state, key bindings, tab logic
      ui.rs                 Root layout: header, tab bar, footer, modals
      ui\dashboard.rs       Dashboard tab (CPU/RAM/GPU/VRAM/temps/FPS)
      ui\steam_panel.rs     Steam tab (library list, game actions, HUD tuning)
      ui\pages.rs           Frames, Processes, Overdrive, System, History, Settings tabs
      ui\components.rs      Shared helpers: styled blocks, badges, progress bars
      config.rs             Profiles, process policy, overlay config, TOML read/write
      i18n.rs               ALL visible text — route every new string here
      steam.rs              Steam library discovery, session tracking, URI commands
      frames.rs             RTSS frame probing, FPS capture
      overlay.rs            RTSS shared-memory OSD backend
      system.rs             Windows state, Overdrive apply/restore actions
      theme.rs              Theme presets, live reload from theme.toml
      game_resolver.rs      Process-to-game resolution scoring
      history.rs            Append-only action/session log (has unit tests)
      metrics.rs            Readiness score, sparkline history, formatting
      hotkeys.rs            Global Windows keyboard hook (Shift+F12)
      doctor.rs             --doctor diagnostic output
  .github\
    workflows\release.yml   CI: build, MSI, GitHub release on tag push
    ISSUE_TEMPLATE\         feature.yml, bug.yml — structured issue forms
    pull_request_template.md Mandatory checklist on every PR
  docs\
    onboarding.md           This file
  packaging\                WiX MSI packaging assets
  dist\                     Build output (gitignored)
  install.ps1               Install / Update / Uninstall script
  install.sh                Wrapper for Git Bash / MSYS users
  build-msi.ps1             Local MSI builder
  dev-watch.ps1             File-watch loop for local dev
```

## Documentation Map

| Document | Location | Purpose |
|---|---|---|
| README | [README.md](../README.md) | Features, installation, configuration, release workflow |
| Onboarding | [docs/onboarding.md](onboarding.md) | This file — setup, commands, contributing workflow |
| UI polish plan | [CRUSH_TUI_PROMPT.md](../CRUSH_TUI_PROMPT.md) | 8-phase TUI visual improvement plan |
| Release workflow | [.github/workflows/release.yml](../.github/workflows/release.yml) | CI pipeline: build, package, publish |

## Contributing Workflow

### Branch strategy

```
master  ──────────────────────────────────────── production releases
          ↑ PR: dev → master (release only)
dev     ──┬─────────────────┬──────────────────── integration branch
           ↑                 ↑
        feature/...      fix/...
```

- All feature branches are cut from `dev`: `git checkout dev && git checkout -b feature/my-thing`
- All PRs target `dev`, not `master`
- `dev → master` PRs represent a production release

### Branch naming

```
feature/<short-slug>    new functionality
fix/<short-slug>        bug fix
chore/<short-slug>      tooling, CI, docs, refactor
```

### Commit format

```
type: short description in present tense

feat: add per-game RTSS preset saving
fix: prevent double-lock on game process detection
docs: update overlay configuration example
chore: bump Cargo dependencies
refactor: extract FPS smoothing into metrics.rs
```

### PR checklist

Every PR must:
- Base branch is `dev`
- `cargo clippy --all-targets --all-features --locked -- -D warnings` passes
- `cargo fmt -- --check` passes
- `cargo test` passes
- Docs updated if behavior or config changed
- Closes the relevant GitHub issue

## Delivery Phases

| Milestone | Status | Scope |
|---|---|---|
| M0: Foundation | **current** | Dev branch, CI, docs, issue workflow, onboarding |
| M1: Core Features | shipped | Steam library, Overdrive profiles, RTSS FPS overlay, fullscreen OSD, 8 themes, bilingual UI |
| M2: Platform Expansion | planned | Epic Games, GOG, Xbox/Game Pass, Battle.net, manual game folders |
| M3: Production Hardening | planned | Per-game profiles, export/import config, full i18n coverage, test suite |

M1 is fully shipped as of v1.3.9. M0 is the current workflow setup. M2 and M3 are roadmap.
