# ⚡ Chaos Game Mode

Optimización extrema para gaming en Windows. Script de PowerShell con **TUI interactiva** que mata procesos innecesarios, detiene servicios del sistema, libera RAM y prioriza Steam al instante.

## Sistema objetivo

| Componente | Especificación |
|------------|---------------|
| CPU | AMD Ryzen 5 5500 (6C/12T) |
| GPU | AMD Radeon RX 550 (4GB) |
| RAM | 16 GB DDR4 |
| SO | Windows 11 Pro |

## Instalación

### Opción 1: Script independiente (recomendado)

```powershell
# Descarga o copia chaosgamemode.ps1 a una carpeta, luego:
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
.\chaosgamemode.ps1          # Abre la TUI interactiva
.\chaosgamemode.ps1 on       # Activar modo juego
.\chaosgamemode.ps1 off      # Restaurar sistema
.\chaosgamemode.ps1 status   # Ver diagnóstico compacto
```

### Opción 2: Perfil de PowerShell (disponible globalmente)

```powershell
# Agrega la funcion a tu perfil
notepad $PROFILE
# Pega el contenido de chaosgamemode.ps1 (sin el bloque param)
# Guarda y recarga:
. $PROFILE
```

## Uso

### `chaosgamemode` (sin argumentos) — TUI interactiva

Ejecuta el script sin parámetros para abrir la **Terminal UI**:

```
╭────────────────────────────────────────────────╮
│   CHAOS GAME MODE                             │
│  Ryzen 5 5500  |  RX 550 4GB  |  16GB RAM      │
╰────────────────────────────────────────────────╯

       [1] Activar modo juego
       [2] Restaurar sistema
       [3] Diagnosticar sistema
       [4] Salir

  ─────────────────────────────────────────────────

    Energia:  Balanceado
    RAM:      3.4GB / 15.9GB (21% libre)
    Steam:    Activo
    Basura:   11281 MB  (chrome 4231MB, dropbox 1612MB...)

  ─────────────────────────────────────────────────

     Opcion [1-4]:
```

- Presiona **1** → Activa modo juego (mata procesos, detiene servicios, prioriza Steam)
- Presiona **2** → Restaura sistema (revive explorer y servicios, plan energía balanceado)
- Presiona **3** → Diagnóstico compacto con barras de uso por proceso
- Presiona **4** → Salir

## Uso

### `chaosgamemode on`

Activa el modo juego:

1. **Plan de energía** → Alto Rendimiento
2. **Mata procesos** → Chrome, Edge, Dropbox, OneDrive, IDMan, qBittorrent,
   Discord, SteelSeries, Epomaker, AnyDesk, WhatsApp, Office, etc.
3. **Detiene servicios** → SysMain, Windows Search, Telemetría, Print Spooler,
   Font Cache, Themes, Push Notifications, Update Orchestrator
4. **Optimiza Steam** → Prioridad alta + 12 núcleos. Si no está abierto, lo lanza.
5. **Libera RAM** → Mata explorer.exe (-300 a -500 MB) + vacía caché del sistema.

### `chaosgamemode off`

Restaura el sistema:

1. Revive **explorer.exe**
2. Reactiva los **servicios** detenidos
3. Vuelve al plan **Balanceado**

> Apps cerradas no se reabren automáticamente.

### `chaosgamemode status`

Diagnóstico **compacto** con procesos agrupados y barras visuales:

```
╭────────────────────────────────────────────────╮
│    DIAGNÓSTICO DEL SISTEMA                    │
╰────────────────────────────────────────────────╯

    Plan de energia:  Alto Rendimiento
    Explorador:        ACTIVO
    Memoria RAM:       3.4GB / 15.9GB (21% libre)
    Steam:             ACTIVO (742 MB)
    Servicios:         4 activos de 9

    Procesos residuales:
   ─────────────────────────────────────────────────
   chrome              4231 MB  42x  ██████████████████████████████ 37%
   dropbox             1612 MB   8x  ███████████░░░░░░░░░░░░░░░░░░░ 14%
   msedgewebview2       853 MB  27x  ██████░░░░░░░░░░░░░░░░░░░░░░░░  8%
   onedrive             142 MB   1x  █░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  1%
   ...

    Total: 11281 MB en 25 tipos de procesos
```

## Procesos eliminados

| Categoría | Procesos |
|-----------|----------|
| **Navegadores** | `chrome*`, `msedge*`, `firefox*`, `opera*`, `brave*`, `vivaldi*` |
| **Cloud Sync** | `dropbox*`, `googledrive*`, `gdrive*`, `onedrive*`, `filecoauth` |
| **Descargas** | `idman*`, `qbittorrent*`, `torrent*`, `transmission*` |
| **Chats** | `discord*`, `slack*`, `teams*`, `zoom*`, `skype*` |
| **Musica** | `spotify*`, `apple*`, `itunes*` |
| **Perifericos** | `steelseries*`, `sonar*`, `epomaker*`, `rapoo*`, `logitech*`, `razer*` |
| **Acceso remoto** | `anydesk*`, `teamviewer*`, `rcclient*`, `rcservice*`, `vnc*` |
| **Mensajeria** | `whatsapp*`, `telegram*`, `signal*` |
| **Office** | `winword*`, `excel*`, `powerpnt*`, `outlook*`, `office*` |
| **File Managers** | `onecommander*`, `files*`, `totalcmd*` |
| **GPU/Overlay** | `radeonsoftware*`, `amdryzenmaster*`, `msiafterburner*` |
| **Windows UI** | `widgets*`, `widgetservice*`, `searchapp*`, `searchhost*` |
| **Monitoreo** | `trafficmonitor*`, `hwmonitor*`, `cpuid*` |
| **Boosters** | `opengameboost*`, `razercortex*` |
| **PDF** | `foxit*`, `acrobat*`, `adobereader*` |

## Servicios detenidos

| Servicio | Funcion |
|----------|---------|
| `SysMain` | Superfetch / SysMain |
| `WSearch` | Windows Search (indexador) |
| `DiagTrack` | Telemetria y diagnostico |
| `Spooler` | Print Spooler (cola de impresion) |
| `FontCache` | Cache de fuentes |
| `PcaSvc` | Program Compatibility Assistant |
| `UsoSvc` | Update Orchestrator |
| `Themes` | Temas de Windows |
| `WpnService` | Windows Push Notifications |

## Personalización

Edita las variables al inicio del script para adaptarlo:

```powershell
# Agrega o quita procesos de la lista negra
$KillList = @(
    'chrome*', 'dropbox*', ...
)

# Agrega o quita servicios a detener
$ServicesToStop = @(
    @{ Name = 'SysMain'; Display = 'SysMain' }
)

# Cambia la ruta de Steam si es diferente
$SteamPaths = @(
    'C:\Program Files (x86)\Steam\steam.exe'
)
```

## Compatibilidad

- Windows 10 / 11 (PowerShell 5.1+ o PowerShell 7+)
- Ejecutar como **usuario normal** (no requiere Admin para matar procesos)
- Para cambiar el **plan de energia** se requieren privilegios de administrador

## Licencia

MIT
