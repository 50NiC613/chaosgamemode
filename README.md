# ⚡ Chaos Game Mode

Optimización extrema para gaming en Windows. Un script de PowerShell que mata procesos innecesarios, detiene servicios del sistema, libera RAM y prioriza Steam al instante.

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
.\chaosgamemode on      # Activar modo juego
.\chaosgamemode off     # Restaurar sistema
.\chaosgamemode status  # Ver diagnósticos
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

### `chaosgamemode on`

Activa el modo juego. Realiza 5 pasos:

1. **Plan de energía** → Cambia a **Alto Rendimiento**
2. **Mata procesos** → Chrome, Edge, Firefox, Dropbox, OneDrive, Google Drive,
   IDMan, qBittorrent, Discord, Slack, Teams, Spotify, SteelSeries, Epomaker,
   Rapoo, Logitech, Razer, AnyDesk, TeamViewer, VNC, WhatsApp, Telegram,
   Office, OneCommander, Radeon Software, Widgets, y mas...
3. **Detiene servicios** → SysMain (Superfetch), Windows Search, Telemetría,
   Print Spooler, Font Cache, Themes, Push Notifications, Update Orchestrator
4. **Optimiza Steam** → Prioridad alta + todos los núcleos.
   Si Steam no está abierto, lo lanza automáticamente.
5. **Libera RAM** → Mata explorer.exe (-300 a -500 MB) y vacía la caché del sistema.

### `chaosgamemode off`

Restaura el sistema:

1. Revive **explorer.exe** (escritorio y barra de tareas)
2. Reactiva los **servicios** que fueron detenidos
3. Vuelve al plan de energía **Balanceado**

> **Nota:** Las aplicaciones cerradas (Chrome, Dropbox, etc.) no se reabren
> automáticamente. Debes abrirlas manualmente o reiniciar el PC.

### `chaosgamemode status`

Muestra un diagnóstico completo del sistema:

```
╔══════════════════════════════════════════════╗
║        CHAOS GAME MODE: STATUS              ║
╚══════════════════════════════════════════════╝
  ⚡ Energia:  ALTO RENDIMIENTO (activo)
  🖥️  Explorer: SUSPENDIDO (modo gamer extremo)
  🧠 RAM:     9.2 GB libres de 15.9 GB (58% libre)
  🔍 Escaneando procesos basura...
    ✅ Ningun proceso basura detectado
  🎮 Steam:   EN EJECUCION
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
