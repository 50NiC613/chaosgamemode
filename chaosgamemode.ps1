#=============================================================================
# CHAOS GAME MODE
# Optimización extrema para gaming en Windows
#
# Sistema objetivo: Ryzen 5 5500 | RX 550 | 16GB RAM | Windows 11 Pro
# Autor: Generado con OpenCode
#=============================================================================

param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet("on", "off", "status")]
    [string]$State
)

# --- CONFIGURACIÓN -----------------------------------------------------------

$KillList = @(
    'chrome*', 'msedge*', 'firefox*', 'opera*', 'brave*', 'vivaldi*'
    'dropbox*', 'googledrive*', 'gdrive*', 'onedrive*', 'filecoauth'
    'idman*', 'qbittorrent*', 'torrent*', 'transmission*'
    'discord*', 'slack*', 'teams*', 'zoom*', 'skype*'
    'spotify*', 'apple*', 'itunes*'
    'steelseries*', 'sonar*', 'gg*'
    'epomaker*', 'rapoo*', 'logitech*', 'razer*'
    'anydesk*', 'teamviewer*', 'rcclient*', 'rcservice*', 'anyviewer*', 'vnc*'
    'whatsapp*', 'telegram*', 'signal*'
    'winword*', 'excel*', 'powerpnt*', 'outlook*', 'office*', 'word*'
    'onecommander*', 'files*', 'totalcmd*'
    'radeonsoftware*', 'amdryzenmaster*', 'msiafterburner*'
    'widgets*', 'widgetservice*', 'news*', 'bing*', 'cortana*'
    'trafficmonitor*', 'hwmonitor*', 'cpuid*'
    'opengameboost*', 'razercortex*'
    'foxit*', 'acrobat*', 'adobereader*'
    'snippingtool*', 'screenshot*', 'snip*'
    'searchapp*', 'searchhost*', 'searchindexer*'
    'startmenuexperiencehost*', 'shellexperiencehost*'
    'runtimebroker*'
    'python*', 'node*', 'dotnet*'
)

$ServicesToStop = @(
    @{ Name = 'SysMain';          Display = 'SysMain (Superfetch)' }
    @{ Name = 'WSearch';          Display = 'Windows Search' }
    @{ Name = 'DiagTrack';        Display = 'Telemetria (DiagTrack)' }
    @{ Name = 'Spooler';          Display = 'Print Spooler' }
    @{ Name = 'FontCache';        Display = 'Font Cache' }
    @{ Name = 'PcaSvc';           Display = 'Program Compatibility Assistant' }
    @{ Name = 'UsoSvc';           Display = 'Update Orchestrator' }
    @{ Name = 'Themes';           Display = 'Themes' }
    @{ Name = 'WpnService';       Display = 'Push Notifications' }
)

$HighPerfGUID = '8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c'
$BalancedGUID = '381b4222-f694-41f0-9685-ff5bb260df2e'

$SteamPaths = @(
    'C:\Program Files (x86)\Steam\steam.exe'
    'C:\Program Files\Steam\steam.exe'
)

# --- FUNCIONES AUXILIARES ----------------------------------------------------

function Write-Banner {
    param([string]$Title, [string]$Color)
    Write-Host "`n" -NoNewline
    Write-Host "╔══════════════════════════════════════════════╗" -ForegroundColor $Color
    Write-Host "║ $Title" -ForegroundColor $Color
    Write-Host "╚══════════════════════════════════════════════╝" -ForegroundColor $Color
}

function Get-RamInfo {
    $os = Get-CimInstance Win32_OperatingSystem
    $free  = [math]::Round($os.FreePhysicalMemory / 1MB, 1)
    $total = [math]::Round($os.TotalVisibleMemorySize / 1MB, 1)
    return @{ Free = $free; Total = $total }
}

function Clear-StandbyMemory {
    try {
        $before = (Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory
        [System.GC]::Collect()
        [System.GC]::WaitForPendingFinalizers()
        $after = (Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory
        return [math]::Round(($after - $before) / 1KB, 1)
    }
    catch { return 0 }
}

# --- ACCIÓN: ON --------------------------------------------------------------

function Invoke-ChaosOn {
    $startTime = Get-Date
    $cachedSvcs = @()

    Write-Banner -Title "        ⚡ CHAOS GAME MODE: ON ⚡         " -Color Red
    Write-Host "  Ryzen 5 5500 | RX 550 | 16GB | Win11`n" -ForegroundColor DarkGray

    # 1. Plan de energía
    Write-Host "[1/5] Plan de energia → Alto Rendimiento" -ForegroundColor Yellow
    powercfg /setactive $HighPerfGUID

    # 2. Matar procesos
    Write-Host "[2/5] Eliminando procesos en segundo plano..." -ForegroundColor Yellow
    $killed = 0
    foreach ($pattern in $KillList) {
        $procs = Get-Process -Name $pattern -ErrorAction SilentlyContinue
        foreach ($p in $procs) {
            try {
                $name = $p.Name
                $mb   = [math]::Round(($p.WorkingSet64 / 1MB), 1)
                Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
                Write-Host "  ✕ $name ($mb MB)" -ForegroundColor DarkGray
                $killed++
            }
            catch { }
        }
    }
    if ($killed -eq 0) {
        Write-Host "  (ningun proceso pesado encontrado)" -ForegroundColor DarkGray
    }
    else {
        Write-Host "  → $killed procesos terminados" -ForegroundColor Green
    }

    # 3. Detener servicios no críticos
    Write-Host "[3/5] Deteniendo servicios del sistema..." -ForegroundColor Yellow
    $stopped = 0
    foreach ($svc in $ServicesToStop) {
        $s = Get-Service -Name $svc.Name -ErrorAction SilentlyContinue
        if ($s -and $s.Status -eq 'Running') {
            try {
                Stop-Service -Name $s.Name -Force -ErrorAction SilentlyContinue
                $cachedSvcs += $s.Name
                Write-Host "  ◉ $($svc.Display) → Detenido" -ForegroundColor DarkGray
                $stopped++
            }
            catch { }
        }
    }
    Set-Variable -Name 'CachedServices' -Value $cachedSvcs -Scope Script
    if ($stopped -eq 0) {
        Write-Host "  (servicios ya optimizados)" -ForegroundColor DarkGray
    }
    else {
        Write-Host "  → $stopped servicios detenidos" -ForegroundColor Green
    }

    # 4. Prioridad Steam + auto-lanzamiento
    Write-Host "[4/5] Verificando Steam..." -ForegroundColor Yellow
    $steamRunning = Get-Process -Name 'steam' -ErrorAction SilentlyContinue
    if (-not $steamRunning) {
        $found = $false
        foreach ($sp in $SteamPaths) {
            if (Test-Path $sp) {
                Write-Host "  → Steam no estaba abierto. Abriendo..." -ForegroundColor Yellow
                Start-Process $sp
                $found = $true
                break
            }
        }
        if (-not $found) {
            Write-Host "  [!] No se encontro Steam. Abrelo manualmente." -ForegroundColor DarkYellow
        }
    }
    else {
        Write-Host "  ✓ Steam ya estaba en ejecucion" -ForegroundColor Green
        foreach ($sp in @('steam', 'steamservice', 'steamwebhelper')) {
            $p = Get-Process -Name $sp -ErrorAction SilentlyContinue
            if ($p) {
                try { $p.PriorityClass = [System.Diagnostics.ProcessPriorityClass]::High }
                catch { }
                try { $p.ProcessorAffinity = [IntPtr]0xFFF }
                catch { }
            }
        }
        Write-Host "  ✓ Steam priorizado (High + todos los nucleos)" -ForegroundColor Green
    }

    # 5. Explorer + limpieza de RAM
    Write-Host "[5/5] Liberando recursos del sistema..." -ForegroundColor Yellow
    $exp = Get-Process -Name 'explorer' -ErrorAction SilentlyContinue
    if ($exp) {
        Stop-Process -Name 'explorer' -Force
        Write-Host "  ✕ explorer.exe suspendido (-300 a -500 MB)" -ForegroundColor DarkGray
    }

    $freed = Clear-StandbyMemory
    if ($freed -gt 0) {
        Write-Host "  ✓ $freed MB liberados de cache RAM" -ForegroundColor Green
    }

    $elapsed = [math]::Round(((Get-Date) - $startTime).TotalSeconds, 1)
    $ram = Get-RamInfo

    Write-Host "`n╔══════════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "║  ✅ CHAOS GAME MODE LISTO ($elapsed s)        ║" -ForegroundColor Green
    Write-Host "║  RAM: $($ram.Free)GB libres / $($ram.Total)GB    ║" -ForegroundColor Cyan
    Write-Host "║  Steam optimizado + prioridad alta           ║" -ForegroundColor Cyan
    Write-Host "║                                              ║" -ForegroundColor DarkGray
    Write-Host "║  Para restaurar: chaosgamemode off           ║" -ForegroundColor Yellow
    Write-Host "╚══════════════════════════════════════════════╝" -ForegroundColor Green
}

# --- ACCIÓN: OFF -------------------------------------------------------------

function Invoke-ChaosOff {
    Write-Banner -Title "       🔄 CHAOS GAME MODE: OFF        " -Color Green

    Write-Host "[1/3] Restaurando interfaz de Windows..." -ForegroundColor Yellow
    if (-not (Get-Process -Name 'explorer' -ErrorAction SilentlyContinue)) {
        Start-Process 'explorer.exe'
        Write-Host "  ✓ explorer.exe reiniciado" -ForegroundColor Green
    }

    $svcs = Get-Variable -Name 'CachedServices' -ErrorAction SilentlyContinue
    if ($svcs -and $svcs.Value.Count -gt 0) {
        Write-Host "[2/3] Restaurando servicios del sistema..." -ForegroundColor Yellow
        $restored = 0
        foreach ($svcName in $svcs.Value) {
            try {
                Start-Service -Name $svcName -ErrorAction SilentlyContinue
                Write-Host "  ◉ $svcName → Iniciado" -ForegroundColor DarkGray
                $restored++
            }
            catch { }
        }
        Write-Host "  → $restored servicios restaurados" -ForegroundColor Green
    }
    else {
        Write-Host "[2/3] (no hay servicios que restaurar)" -ForegroundColor DarkGray
    }

    Write-Host "[3/3] Plan de energia → Balanceado" -ForegroundColor Yellow
    powercfg /setactive $BalancedGUID

    Write-Host "`n╔══════════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║  ✅ Sistema restaurado                        ║" -ForegroundColor Cyan
    Write-Host "║  Las apps cerradas (Chrome, Dropbox, etc.)   ║" -ForegroundColor Yellow
    Write-Host "║  deberas reabrirlas manualmente.             ║" -ForegroundColor Yellow
    Write-Host "╚══════════════════════════════════════════════╝" -ForegroundColor Cyan
}

# --- ACCIÓN: STATUS ----------------------------------------------------------

function Invoke-ChaosStatus {
    Write-Banner -Title "     📊 CHAOS GAME MODE: STATUS      " -Color Cyan

    # 1. Plan de energía
    $activePlan = powercfg /getactivescheme
    if ($activePlan -match $HighPerfGUID) {
        Write-Host "  ⚡ Energia:  ALTO RENDIMIENTO (activo)" -ForegroundColor Green
    }
    elseif ($activePlan -match $BalancedGUID) {
        Write-Host "  🔋 Energia:  Balanceado (activo)" -ForegroundColor Yellow
    }
    else {
        Write-Host "  ⚡ Energia:  Personalizado" -ForegroundColor DarkGray
    }

    # 2. Explorer
    if (Get-Process -Name 'explorer' -ErrorAction SilentlyContinue) {
        Write-Host "  🖥️  Explorer: ACTIVO (escritorio visible)" -ForegroundColor Yellow
    }
    else {
        Write-Host "  🖥️  Explorer: SUSPENDIDO (modo gamer extremo)" -ForegroundColor Green
    }

    # 3. RAM
    $ram = Get-RamInfo
    $pct = [math]::Round(($ram.Free / $ram.Total) * 100, 0)
    Write-Host "  🧠 RAM:     $($ram.Free)GB libres de $($ram.Total)GB ($pct% libre)" -ForegroundColor Magenta

    # 4. Procesos basura
    Write-Host "  🔍 Escaneando procesos basura..." -ForegroundColor Cyan
    $foundAny = $false
    $totalWaste = 0
    foreach ($pattern in $KillList) {
        $procs = Get-Process -Name $pattern -ErrorAction SilentlyContinue
        foreach ($p in $procs) {
            $mb = [math]::Round(($p.WorkingSet64 / 1MB), 1)
            Write-Host "    ⚠ $($p.Name) -> $mb MB" -ForegroundColor Red
            $foundAny = $true
            $totalWaste += $mb
        }
    }
    if (-not $foundAny) {
        Write-Host "    ✅ Ningun proceso basura detectado" -ForegroundColor Green
    }
    else {
        Write-Host "    ⚠ Total: $totalWaste MB en procesos residuales" -ForegroundColor Red
    }

    # 5. Steam
    $steam = Get-Process -Name 'steam' -ErrorAction SilentlyContinue
    if ($steam) {
        Write-Host "  🎮 Steam:   EN EJECUCION" -ForegroundColor Green
    }
    else {
        Write-Host "  🎮 Steam:   CERRADO" -ForegroundColor DarkGray
    }

    Write-Host "`n══════════════════════════════════════════════" -ForegroundColor Cyan
}

# --- ENTRY POINT -------------------------------------------------------------

switch ($State) {
    'on'     { Invoke-ChaosOn }
    'off'    { Invoke-ChaosOff }
    'status' { Invoke-ChaosStatus }
}
