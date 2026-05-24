# Prompt para Crush: interfaz definitiva de Chaos Game Mode

Usa este archivo como prompt completo para el agente Crush. El objetivo no es
reescribir la app desde cero, sino llevar la TUI Rust existente a un nivel
profesional de producto, respetando la arquitectura y las funciones actuales.

## Rol

Eres Crush actuando como ingeniero senior de Rust, Ratatui y diseno de TUIs
modernas estilo Charmbracelet, Gum, Glow y Crush. Tu especialidad es crear
interfaces de terminal limpias, expresivas, rapidas y mantenibles.

Trabaja en:

```powershell
D:\Dev\chaosgamemode\tui-rs
```

## Objetivo

Convertir la TUI Rust de Chaos Game Mode en la interfaz terminal definitiva:
pulida, responsive, consistente, segura de usar y visualmente memorable.

La app ya existe y ya tiene una arquitectura modular. No conviertas esto en una
reescritura completa. Evoluciona lo que ya hay.

## Contexto actual de la app

El proyecto es una TUI nativa de Windows para optimizacion gaming. Usa Rust,
Ratatui, Crossterm y Sysinfo. La app permite inspeccionar telemetria, detectar
procesos pesados, clasificar procesos del perfil activo, escanear Steam, lanzar
juegos y ejecutar Overdrive con confirmacion previa.

Arquitectura actual:

```text
tui-rs/src/
  main.rs              entrada minima; llama a app::run()
  app.rs               estado, loop, eventos, acciones, tabs y confirmaciones
  config.rs            config.toml, perfiles, procesos protegidos/ocultos y timings
  system.rs            Windows, procesos, servicios, energia y SystemState
  steam.rs             escaneo Steam, lanzamiento de juegos y sesiones
  history.rs           historial append-only de acciones y sesiones
  theme.rs             tema gruv-neon y recarga en vivo de theme.toml
  metrics.rs           porcentajes, readiness, historiales y formato
  ui.rs                layout raiz, header, tabs, footer, confirmacion y output
  ui/components.rs     widgets reutilizables
  ui/dashboard.rs      dashboard de telemetria
  ui/steam_panel.rs    pantalla Steam
  ui/pages.rs          procesos, Overdrive y sistema
  i18n.rs              textos localizados ES/EN para UI, logs y estados
```

Dependencias relevantes:

```toml
ratatui = "0.29"
crossterm = "0.28"
sysinfo = "0.32"
serde = { version = "1", features = ["derive"] }
toml = "0.8"
```

Funciones que ya existen y deben conservarse:

- Telemetria en segundo plano.
- Historial de CPU, RAM y bloat.
- Perfiles `safe`, `balanced` y `aggressive`.
- `config.toml` editable y persistencia de cambios de procesos.
- `theme.toml` con recarga en vivo.
- Tabs: Dashboard, Steam, Frames, Procesos, Overdrive, Sistema, Historial y
  Ajustes.
- Navegacion por mouse en la barra de tabs; los hitboxes deben coincidir con
  el layout renderizado.
- Clasificador de procesos con `TARGET`, `KEEP`, `WATCH` y `HIDDEN`.
- Filtro de procesos con `/`.
- Vista de procesos ocultos con `V`.
- Confirmacion antes de Overdrive.
- Output/action log con scroll.
- Escaneo de bibliotecas Steam.
- Lanzamiento Steam normal y con Overdrive.
- Sesiones Steam con timer e historial.
- Idioma configurable en `[ui] language = "es"` o `"en"`.
- Tema hacker inspirado en Mr. Robot, junto a Cyberpunk, Gruvbox, Tokyo Night,
  Catppuccin/Mocha y otros presets.

## Reglas de trabajo

- No recrees modulos que ya existen.
- No muevas codigo solo por estetica.
- No elimines features actuales.
- No confirmes Overdrive durante pruebas manuales.
- Puedes abrir el preview/modal de Overdrive, pero cancela con `Esc`, `N` o `Q`.
- Mantener codigo idiomatico Rust.
- Preferir borrowing sobre clones innecesarios.
- No usar `unwrap()` o `expect()` en produccion si hay alternativa limpia.
- Extraer helpers solo cuando reduzcan duplicacion real.
- Mantener compatibilidad con Windows.
- Mantener compatibilidad con `theme.toml`, `config.toml`, `CHAOS_THEME`,
  `CHAOS_CONFIG`, `CHAOS_HISTORY` y `STEAM_DIR`.
- Mantener el sistema de i18n centralizado en `src/i18n.rs`; no introducir
  nuevos textos visibles hardcodeados en pantallas, modales o action logs.
- Al final, ejecutar `cargo fmt`, `cargo test` y
  `cargo clippy --all-targets --all-features -- -D warnings`.

## Baseline inicial

Antes de editar, ejecuta:

```powershell
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Si fallan, entiende primero si el fallo ya venia del baseline o si es causado
por tus cambios. El estado esperado del proyecto actual es que tests y clippy
pasen.

## Fase 1: auditoria visual y tecnica

Lee estos archivos antes de editar:

```text
src/app.rs
src/theme.rs
src/ui.rs
src/ui/components.rs
src/ui/dashboard.rs
src/ui/pages.rs
src/ui/steam_panel.rs
src/i18n.rs
```

Identifica:

- Duplicacion de keycaps, badges, bloques, titulos y label/value pairs.
- Panels que se ven demasiado cargados.
- Bordes pesados o inconsistentes.
- Lugares donde el texto toca bordes.
- Layouts que pueden romperse en terminales pequenas.
- Estados vacios o de loading poco claros.
- Footer/header sobrecargados.
- Strings visibles que no pasen por `Language`.

No hagas una refactorizacion grande hasta entender como fluyen `App`, `Screen`,
`Tab`, `PendingAction`, `SystemState`, `SteamLibrary` y `Theme`.

## Fase 2: sistema visual Tuisthetics

Crear una capa visual consistente en `src/ui/components.rs`.

Prioriza helpers reutilizables como:

```text
panel_block
danger_block
modal_block
keycap
metric_label
metric_value
status_badge
selected_row_style
muted_text
accent_text
panel_title
centered_rect
```

Objetivos visuales:

- Estetica premium de terminal, inspirada en Charmbracelet.
- Menos ruido neon, mas contraste elegante.
- Bordes `Rounded` o `Thick`; evitar `Double`.
- Espaciado interno en blocks y paragraphs cuando Ratatui lo permita.
- Labels en color muted.
- Valores importantes en bold y color de acento.
- Estados criticos con color danger claro, sin gritar todo el tiempo.
- Paleta curada usando `Color::Rgb`, compatible con `theme.toml`.

La identidad debe seguir siendo Chaos Game Mode: energia, rendimiento, gaming,
Overdrive, Steam y Windows tuning. Pero debe sentirse como herramienta seria,
no demo caotica.

## Fase 3: layout responsive

La TUI debe verse bien en:

```text
80x24
100x30
120x40
pantalla ancha
```

Criterios:

- Nada de texto incoherentemente solapado.
- No confiar en alturas fijas fragiles.
- Usar `saturating_sub`, constraints conservadoras y truncamiento cuando toque.
- En terminal pequena, preferir textos cortos y panels mas densos.
- Header compacto: nombre, readiness, perfil, sesion y metricas resumidas.
- Footer contextual: mostrar solo teclas relevantes para la tab actual.

## Fase 4: tabs y navegacion

Mejorar `Tabs` en `src/ui.rs`:

- La tab activa debe parecer seleccionada/presionada.
- Usar fondo de acento con texto oscuro, o underline/bold si queda mejor.
- Mantener navegacion existente:
  - `Tab` / flecha derecha: siguiente tab.
  - `Shift+Tab` / flecha izquierda: tab anterior.
  - `Q` / `Esc`: salir desde monitor.

No cambies nombres de tabs salvo para hacerlos mas claros:

```text
Dashboard, Steam, Frames, Procesos, Overdrive, Sistema, Historial, Ajustes
```

Si tocas el render de tabs, valida tambien el click de mouse sobre cada tab.
Un bug anterior desplazaba `Sistema`, `Historial` y `Ajustes`; los calculos de
`Tab::nav_slots` y `Tab::from_nav_column` deben mantenerse alineados.

## Fase 5: modal real de confirmacion Overdrive

La app ya tiene `Screen::Confirm` y `PendingAction`. Mejora la presentacion sin
romper la logica.

Comportamiento deseado:

- Cuando `Screen::Confirm` este activo, renderizar primero el monitor de fondo.
- Encima, dibujar un backdrop oscuro o atenuado.
- Usar `ratatui::widgets::Clear` en un rect centrado.
- Dibujar un modal centrado con borde danger/accent.
- Mostrar el preview existente con scroll.
- Mantener la lista de procesos objetivo, memoria, instancias y exe path.

Controles del modal:

```text
Y o Enter       confirmar
N, Esc o Q      cancelar
Up/Down         scroll fino
PageUp/PageDown scroll por pagina
Home/End        inicio/final
```

Actualiza los textos de ayuda para que hablen de `Y/N`, no solo `Enter/Esc`.

Importante:

- No ejecutes Overdrive mientras pruebas.
- Si haces una prueba manual, abre el modal y cancela.

## Fase 6: pulido por pantalla

### Dashboard

- Debe responder rapido a lectura visual.
- Readiness debe ser el centro de decision.
- CPU, RAM y bloat deben tener jerarquia clara.
- Sparklines compactas con titulos legibles.
- Heatmap de procesos menos ruidoso, mas tipo tabla.

### Steam

- Lista de juegos con seleccion fuerte pero limpia.
- Empty/scanning states profesionales.
- Panel de juego seleccionado claro.
- Panel de sesion activa destacado.
- Acciones `Enter`, `L`, `S` y `E` como keycaps consistentes.

### Frames

- Debe mostrar FPS, promedio, 1% low, frame time, samples y target actual.
- Debe localizar estados de RTSS y respetar `[ui] language`.
- `R` debe refrescar deteccion de RTSS y `E` cerrar la sesion activa.

### Procesos

- Lista como tabla compacta.
- Status badge claro: `TARGET`, `KEEP`, `WATCH`, `HIDDEN`.
- Seleccion visible sin romper alineacion.
- Filtro `/` visible y compacto.
- Vista hidden/actionable clara.
- Detalle del proceso debe priorizar nombre, memoria, estado y exe path.

### Overdrive

- Debe sentirse como consola de accion segura.
- Comandos como botones/keycaps.
- Readiness y estado vivo como senales principales.
- Recordar que Overdrive siempre pasa por preview/modal.

### Sistema

- Telemetria como pares label/value.
- Menos texto decorativo.
- Mostrar CPU, RAM, energia, explorer, Steam y servicios con claridad.

### Output

- Action log legible y scrolleable.
- Titulos con rango de scroll si aplica.
- Hint inferior compacto.

## Fase 7: tema

Revisar `src/theme.rs` y `theme.toml`.

Opciones aceptables:

- Pulir Gruvbox Neon actual.
- Migrar a una paleta tipo Catppuccin Mocha.
- Mantener nombres de campos de `Theme` para no romper demasiado codigo.

No uses colores puros tipo rojo/verde/azul primarios. Usa RGB curados.

El resultado debe tener:

- fondo profundo
- paneles ligeramente diferenciados
- foreground calido o claro
- muted legible
- accent principal
- accent secundario
- danger
- success
- warning

El preset `hacker` debe mantenerse: negro profundo, verdes terminal, acentos
tipo consola de operaciones y legibilidad alta sin convertir toda la app en una
paleta de un solo verde.

## Fase 7.5: i18n ES/EN

La app ya soporta idioma desde `config.toml`:

```toml
[ui]
language = "es" # o "en"
```

Requisitos:

- Todo texto visible nuevo debe pasar por `src/i18n.rs` o por helpers de
  localizacion centralizados.
- Mantener sentido natural en espanol e ingles; no hacer traducciones literales
  raras.
- El espanol debe sonar a gamer hispano viviendo en Estados Unidos, no a
  doblaje neutro ni a espanol LatAm generico. Es valido usar Spanglish tecnico
  cuando sea lo natural: `READY CHECK`, `GAME READY`, `NEEDS CLEANUP`,
  `TARGETS`, `KILL LIST`, `BLOAT`, `scan`, `launch`, `timer`, `profile`,
  `Power plan`, `WATCH`.
- Evitar palabras que suenan forzadas en esta app: `preparacion`, `residuo`,
  `requiere limpieza`, `a cerrar`, `carga removible`, `objetivos residuales`.
- Los acronimos tecnicos universales (`CPU`, `GPU`, `FPS`, `RAM`, `APPID`,
  `EXE`) pueden quedarse iguales.
- Los logs de Overdrive, Restore, Steam, sesiones, historial, modales y estados
  vacios deben respetar el idioma.
- Los estados generados por subsistemas (`SteamLibrary.status`,
  `FrameMetrics.status`, `HardwareState.status`, `FrameProbe.status`,
  `config.status`, `theme_status`) deben mostrarse localizados mediante helpers
  centralizados antes de renderizar.

## Fase 8: pruebas y verificacion

Ejecutar:

```powershell
cargo fmt
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Luego hacer prueba visual segura:

```powershell
cargo run
```

Checklist manual:

- La app abre sin panic.
- Dashboard se ve correcto.
- Cambiar tabs con Tab y flechas.
- Steam no se rompe aunque no haya juegos detectados.
- Procesos navega con Up/Down.
- Filtro `/` funciona.
- Modal abre con `1` o `Space`.
- Modal cancela con `N` o `Esc`.
- Output sigue navegable.
- Salir deja la terminal limpia.

No confirmar Overdrive durante esta verificacion.

## Entrega esperada

Al terminar, responde con:

- Archivos modificados.
- Resumen de cambios visuales.
- Resumen de cambios arquitectonicos, si hubo.
- Resultado de `cargo fmt`, `cargo test` y `cargo clippy`.
- Cualquier riesgo o follow-up pendiente.

## Criterios de aceptacion

La tarea se considera completa solo si:

- La TUI mantiene todas las funciones actuales.
- La interfaz se ve mas limpia y profesional.
- El modal Overdrive es realmente modal y centrado.
- Las tabs tienen estado activo claro.
- La app se adapta a terminales chicas y grandes.
- `theme.toml` sigue funcionando con hot reload.
- `config.toml` sigue funcionando.
- Cambiar `[ui] language` entre `es` y `en` cambia tabs, footer, pantallas,
  modales, action logs y estados principales sin mezcla rara de idiomas.
- Tests y clippy pasan.
