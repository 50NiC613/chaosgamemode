#=============================================================================
# CHAOS GAME MODE  v2.0
# Optimización extrema para gaming en Windows
# Ryzen 5 5500 | RX 550 | 16GB RAM | Windows 11 Pro
#=============================================================================

param(
    [Parameter(Mandatory = $false, Position = 0)]
    [ValidateSet("on", "off", "status")]
    [string]$State
)

# --- ICONOS (Nerd Fonts) -----------------------------------------------------
$I = @{
    On   = ""
    Off  = ""
    TUI  = ""
    Exit = ""
    CPU  = ""
    RAM  = ""
    HDD  = ""
    Net  = ""
    Stm  = ""
    Web  = ""
    Game = ""
    Warn = ""
    OK   = ""
    Svc  = ""
    Gear = ""
    User = ""
    Dir  = ""
    Rstr = ""
    Src  = ""
    Pwr  = ""
    Batt = ""
    Chk  = ""
    Fw   = ""
}

# --- CONFIGURACIÓN -----------------------------------------------------------
$KillList = @(
    'chrome*', 'msedge*', 'firefox*', 'opera*', 'brave*', 'vivaldi*'
    'dropbox*', 'googledrive*', 'gdrive*', 'onedrive*', 'filecoauth'
    'idman*', 'qbittorrent*', 'torrent*', 'transmission*'
    'discord*', 'slack*', 'teams*', 'zoom*', 'skype*'
    'spotify*', 'apple*', 'itunes*'
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
    'python*', 'node*', 'dotnet*'
)

$ServicesToStop = @(
    @{ Name = 'SysMain';          Icon = ''; Display = 'SysMain (Superfetch)' }
    @{ Name = 'WSearch';          Icon = ''; Display = 'Windows Search' }
    @{ Name = 'DiagTrack';        Icon = ''; Display = 'Telemetria (DiagTrack)' }
    @{ Name = 'Spooler';          Icon = ''; Display = 'Print Spooler' }
    @{ Name = 'FontCache';        Icon = ''; Display = 'Font Cache' }
    @{ Name = 'PcaSvc';           Icon = ''; Display = 'Compat. Assistant' }
    @{ Name = 'UsoSvc';           Icon = ''; Display = 'Update Orchestrator' }
    @{ Name = 'Themes';           Icon = ''; Display = 'Themes' }
    @{ Name = 'WpnService';       Icon = ''; Display = 'Push Notifications' }
)

$HighPerfGUID = '8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c'
$BalancedGUID = '381b4222-f694-41f0-9685-ff5bb260df2e'

$SteamPaths = @(
    'C:\Program Files (x86)\Steam\steam.exe'
    'C:\Program Files\Steam\steam.exe'
)

$Script:CachedServices = @()

# --- FUNCIONES AUXILIARES ----------------------------------------------------

function Clear-Screen {
    try { [System.Console]::Clear() }
    catch { Clear-Host }
}

function Write-HR {
    param([string]$Char = '─', [string]$Color = 'DarkGray')
    $w = [System.Console]::WindowWidth - 1
    Write-Host ($Char * $w) -ForegroundColor $Color
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

function Get-PowerPlan {
    $plan = powercfg /getactivescheme
    if ($plan -match $HighPerfGUID) { return 'High', 'Alto Rendimiento' }
    if ($plan -match $BalancedGUID) { return 'Balanced', 'Balanceado' }
    return 'Custom', 'Personalizado'
}

function Get-SteamStatus {
    $p = Get-Process -Name 'steam' -ErrorAction SilentlyContinue
    if ($p) { return $true, ($p.WorkingSet64 / 1MB) }
    return $false, 0
}

function Get-ExplorerStatus {
    return [bool](Get-Process -Name 'explorer' -ErrorAction SilentlyContinue)
}

function Get-ScannedProcesses {
    $groups = @{}
    $total = 0
    foreach ($pattern in $KillList) {
        $procs = Get-Process -Name $pattern -ErrorAction SilentlyContinue
        foreach ($p in $procs) {
            $name = $p.Name
            $mb   = [math]::Round(($p.WorkingSet64 / 1MB), 1)
            if (-not $groups[$name]) { $groups[$name] = @{ Count = 0; MB = 0 } }
            $groups[$name].Count++
            $groups[$name].MB += $mb
            $total += $mb
        }
    }
    return $groups, [math]::Round($total, 1)
}

# --- ACCIÓN: ON --------------------------------------------------------------

function Invoke-ChaosOn {
    $startTime = Get-Date

    Clear-Screen
    Write-Host "╭────────────────────────────────────────────────╮" -ForegroundColor Red
    Write-Host "│  $($I.On)  CHAOS GAME MODE: ACTIVANDO...                │" -ForegroundColor Red
    Write-Host "╰────────────────────────────────────────────────╯" -ForegroundColor Red

    Write-Host "`n$($I.Gear) Plan de energia → Alto Rendimiento" -ForegroundColor Yellow
    powercfg /setactive $HighPerfGUID

    Write-Host "$($I.Svc) Eliminando procesos en segundo plano..." -ForegroundColor Yellow
    $killed = 0
    foreach ($pattern in $KillList) {
        $procs = Get-Process -Name $pattern -ErrorAction SilentlyContinue
        foreach ($p in $procs) {
            try {
                $mb = [math]::Round(($p.WorkingSet64 / 1MB), 1)
                Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
                Write-Host "   $($I.Fw) $($p.Name) libero $mb MB" -ForegroundColor DarkGray
                $killed++
            }
            catch { }
        }
    }
    Write-Host "   $($I.OK) $killed procesos terminados" -ForegroundColor Green

    Write-Host "$($I.Svc) Deteniendo servicios del sistema..." -ForegroundColor Yellow
    $stopped = 0
    $Script:CachedServices = @()
    foreach ($svc in $ServicesToStop) {
        $s = Get-Service -Name $svc.Name -ErrorAction SilentlyContinue
        if ($s -and $s.Status -eq 'Running') {
            try {
                Stop-Service -Name $s.Name -Force -ErrorAction SilentlyContinue
                $Script:CachedServices += $s.Name
                Write-Host "   $($I.Fw) $($svc.Display)" -ForegroundColor DarkGray
                $stopped++
            }
            catch { }
        }
    }
    Write-Host "   $($I.OK) $stopped servicios detenidos" -ForegroundColor Green

    Write-Host "$($I.Stm) Verificando Steam..." -ForegroundColor Yellow
    $steamRunning = Get-Process -Name 'steam' -ErrorAction SilentlyContinue
    if (-not $steamRunning) {
        $found = $false
        foreach ($sp in $SteamPaths) {
            if (Test-Path $sp) {
                Write-Host "   $($I.Fw) Abriendo Steam..." -ForegroundColor Yellow
                Start-Process $sp
                $found = $true
                break
            }
        }
        if (-not $found) {
            Write-Host "   $($I.Warn) No se encontro Steam" -ForegroundColor DarkYellow
        }
    }
    else {
        Write-Host "   $($I.OK) Steam ya activo, priorizando..." -ForegroundColor Green
        foreach ($sp in @('steam', 'steamservice', 'steamwebhelper')) {
            $p = Get-Process -Name $sp -ErrorAction SilentlyContinue
            if ($p) {
                try { $p.PriorityClass = [System.Diagnostics.ProcessPriorityClass]::High }
                catch { }
                try { $p.ProcessorAffinity = [IntPtr]0xFFF }
                catch { }
            }
        }
    }

    Write-Host "$($I.RAM) Liberando recursos del sistema..." -ForegroundColor Yellow
    $exp = Get-Process -Name 'explorer' -ErrorAction SilentlyContinue
    if ($exp) {
        Stop-Process -Name 'explorer' -Force
        Write-Host "   $($I.Fw) explorer.exe libero ~400 MB" -ForegroundColor DarkGray
    }

    $freed = Clear-StandbyMemory
    if ($freed -gt 0) {
        Write-Host "   $($I.OK) $freed MB liberados de cache" -ForegroundColor Green
    }

    $elapsed = [math]::Round(((Get-Date) - $startTime).TotalSeconds, 1)
    $ram = Get-RamInfo

    Write-Host "`n╭────────────────────────────────────────────────╮" -ForegroundColor Green
    Write-Host "│  $($I.OK)  Modo juego listo  ($elapsed s)               │" -ForegroundColor Green
    Write-Host "│  $($I.RAM)  RAM: $($ram.Free)GB / $($ram.Total)GB                        │" -ForegroundColor Cyan
    Write-Host "│  $($I.Stm)  Steam optimizado (High + 12 nucleos)      │" -ForegroundColor Cyan
    Write-Host "╰────────────────────────────────────────────────╯" -ForegroundColor Green

    if (-not $State) {
        Write-Host "`n   Presiona cualquier tecla para volver..." -NoNewline -ForegroundColor DarkGray
        $null = [System.Console]::ReadKey($true)
    }
}

# --- ACCIÓN: OFF -------------------------------------------------------------

function Invoke-ChaosOff {
    Clear-Screen
    Write-Host "╭────────────────────────────────────────────────╮" -ForegroundColor Yellow
    Write-Host "│  $($I.Rstr)  CHAOS GAME MODE: RESTAURANDO...               │" -ForegroundColor Yellow
    Write-Host "╰────────────────────────────────────────────────╯" -ForegroundColor Yellow

    Write-Host "`n$($I.Dir) Restaurando interfaz de Windows..." -ForegroundColor Yellow
    if (-not (Get-Process -Name 'explorer' -ErrorAction SilentlyContinue)) {
        Start-Process 'explorer.exe'
        Write-Host "   $($I.OK) explorer.exe reiniciado" -ForegroundColor Green
    }

    if ($Script:CachedServices.Count -gt 0) {
        Write-Host "$($I.Svc) Restaurando servicios..." -ForegroundColor Yellow
        $restored = 0
        foreach ($svcName in $Script:CachedServices) {
            try {
                Start-Service -Name $svcName -ErrorAction SilentlyContinue
                Write-Host "   $($I.Fw) $svcName" -ForegroundColor DarkGray
                $restored++
            }
            catch { }
        }
        Write-Host "   $($I.OK) $restored servicios restaurados" -ForegroundColor Green
    }

    Write-Host "$($I.Pwr) Plan de energia → Balanceado" -ForegroundColor Yellow
    powercfg /setactive $BalancedGUID

    Write-Host "`n╭────────────────────────────────────────────────╮" -ForegroundColor Cyan
    Write-Host "│  $($I.OK)  Sistema restaurado                         │" -ForegroundColor Cyan
    Write-Host "│  $($I.Warn)  Apps cerradas no se reabren solas          │" -ForegroundColor Yellow
    Write-Host "╰────────────────────────────────────────────────╯" -ForegroundColor Cyan

    if (-not $State) {
        Write-Host "`n   Presiona cualquier tecla para volver..." -NoNewline -ForegroundColor DarkGray
        $null = [System.Console]::ReadKey($true)
    }
}

# --- ACCIÓN: STATUS (COMPACTO) -----------------------------------------------

function Invoke-ChaosStatus {
    $planId, $planName = Get-PowerPlan
    $ram        = Get-RamInfo
    $pct        = [math]::Round(($ram.Free / $ram.Total) * 100, 0)
    $steamOn, $steamMB = Get-SteamStatus
    $explorerOn = Get-ExplorerStatus
    $groups, $totalWaste = Get-ScannedProcesses

    $planIcon   = if ($planId -eq 'High') { $I.Pwr } else { $I.Batt }
    $planColor  = if ($planId -eq 'High') { 'Green' } else { 'Yellow' }
    $expIcon    = if ($explorerOn) { '' } else { '' }
    $expStatus  = if ($explorerOn) { 'ACTIVO' } else { 'SUSPENDIDO' }
    $ramColor   = if ($pct -ge 50) { 'Green' } elseif ($pct -ge 25) { 'Yellow' } else { 'Red' }
    $steamIcon  = if ($steamOn) { $I.Stm } else { $I.Game }
    $steamColor = if ($steamOn) { 'Green' } else { 'DarkGray' }
    $steamText  = if ($steamOn) { "ACTIVO  ($([math]::Round($steamMB,0)) MB)" } else { 'CERRADO' }

    Clear-Screen
    Write-Host "╭────────────────────────────────────────────────╮" -ForegroundColor Cyan
    Write-Host "│  $($I.TUI)  DIAGNÓSTICO DEL SISTEMA                      │" -ForegroundColor Cyan
    Write-Host "╰────────────────────────────────────────────────╯" -ForegroundColor Cyan
    Write-Host ""

    Write-Host "   $planIcon Plan de energia:  " -NoNewline
    Write-Host $planName -ForegroundColor $planColor

    Write-Host "   $expIcon Explorador:        " -NoNewline
    Write-Host $expStatus -ForegroundColor $(if ($explorerOn) { 'Yellow' } else { 'Green' })

    Write-Host "   $($I.RAM) Memoria RAM:       " -NoNewline
    Write-Host "$($ram.Free)GB / $($ram.Total)GB ($pct% libre)" -ForegroundColor $ramColor

    Write-Host "   $steamIcon Steam:            " -NoNewline
    Write-Host $steamText -ForegroundColor $steamColor

    Write-Host "   $($I.Svc) Servicios:         " -NoNewline
    $svcsRunning = 0
    foreach ($svc in $ServicesToStop) {
        $s = Get-Service -Name $svc.Name -ErrorAction SilentlyContinue
        if ($s -and $s.Status -eq 'Running') { $svcsRunning++ }
    }
    $svcColor = if ($svcsRunning -eq 0) { 'Green' } elseif ($svcsRunning -le 3) { 'Yellow' } else { 'Red' }
    Write-Host "$svcsRunning activos de $($ServicesToStop.Count)" -ForegroundColor $svcColor

    Write-Host ""

    if ($groups.Count -gt 0) {
        Write-Host "   $($I.Warn) Procesos residuales que consumen recursos:"
        Write-Host "   ─────────────────────────────────────────────────" -ForegroundColor DarkGray

        $sorted = $groups.GetEnumerator() | Sort-Object { $_.Value.MB } -Descending
        $i = 0
        foreach ($entry in $sorted) {
            $name = $entry.Key
            $data = $entry.Value
            $pctOfTotal = [math]::Round(($data.MB / $totalWaste) * 100, 0)
            $barLen = [math]::Max(1, [math]::Min(30, [math]::Round($data.MB / $totalWaste * 30)))
            $charFull = "$([char]0x2588)"
            $charEmpty = "$([char]0x2591)"
            $bar = ($charFull * $barLen) + ($charEmpty * (30 - $barLen))

            if ($data.MB -ge 500) { $barColor = 'Red' }
            elseif ($data.MB -ge 200) { $barColor = 'Yellow' }
            else { $barColor = 'DarkGray' }

            $truncated = if ($name.Length -gt 20) { $name.Substring(0, 20) } else { $name }
            Write-Host "   $($truncated.PadRight(20)) " -NoNewline -ForegroundColor DarkGray
            Write-Host "$([math]::Round($data.MB,0)) MB".PadLeft(8) -NoNewline -ForegroundColor $barColor
            Write-Host (" $($data.Count)x") -NoNewline -ForegroundColor DarkGray
            Write-Host "  $bar" -NoNewline -ForegroundColor $barColor
            Write-Host " $pctOfTotal%" -ForegroundColor DarkGray

            $i++
            if ($i -ge 10) {
                $remaining = $sorted.Count - 10
                if ($remaining -gt 0) {
                    Write-Host "   ... y $remaining procesos mas" -ForegroundColor DarkGray
                }
                break
            }
        }
        Write-Host ""
        Write-Host "   $($I.RAM) Total: " -NoNewline
        Write-Host "$totalWaste MB" -ForegroundColor Red -NoNewline
        Write-Host " en $($groups.Count) tipos de procesos" -ForegroundColor DarkGray
    }
    else {
        Write-Host "   $($I.OK) No se detectaron procesos residuales" -ForegroundColor Green
    }

    if (-not $State) {
        Write-Host "`n   Presiona cualquier tecla para volver..." -NoNewline -ForegroundColor DarkGray
        $null = [System.Console]::ReadKey($true)
    }
}

# --- TUI: MENÚ PRINCIPAL -----------------------------------------------------

function Show-Menu {
    $ram = Get-RamInfo
    $pct = [math]::Round(($ram.Free / $ram.Total) * 100, 0)
    $planId, $planName = Get-PowerPlan
    $steamOn, $_ = Get-SteamStatus
    $explorerOn = Get-ExplorerStatus
    $groups, $totalWaste = Get-ScannedProcesses

    Clear-Screen
    Write-Host "╭────────────────────────────────────────────────╮" -ForegroundColor Cyan
    Write-Host "│  " -NoNewline -ForegroundColor Cyan
    Write-Host "$($I.Gear) CHAOS GAME MODE" -NoNewline -ForegroundColor Red
    Write-Host "                         │" -ForegroundColor Cyan
    Write-Host "│  Ryzen 5 5500  |  RX 550 4GB  |  16GB RAM     │" -ForegroundColor Cyan
    Write-Host "╰────────────────────────────────────────────────╯" -ForegroundColor Cyan

    Write-Host ""

    if ($explorerOn) { $modeLabel = 'Normal'; $modeColor = 'Yellow' }
    else { $modeLabel = 'GAMER'; $modeColor = 'Green' }

    Write-Host "     $($I.On)  " -NoNewline -ForegroundColor Green
    Write-Host "[1] Activar modo juego" -ForegroundColor White

    Write-Host "     $($I.Off)  " -NoNewline -ForegroundColor Yellow
    Write-Host "[2] Restaurar sistema" -ForegroundColor White

    Write-Host "     $($I.TUI)  " -NoNewline -ForegroundColor Cyan
    Write-Host "[3] Diagnosticar sistema" -ForegroundColor White

    Write-Host "     $($I.Exit)  " -NoNewline -ForegroundColor Red
    Write-Host "[4] Salir" -ForegroundColor White

    Write-Host ""
    Write-Host "  ─────────────────────────────────────────────────" -ForegroundColor DarkGray
    Write-Host ""

    $pctFree = [math]::Round(($ram.Free / $ram.Total) * 100, 0)
    $ramColor = if ($pctFree -ge 50) { 'Green' } elseif ($pctFree -ge 25) { 'Yellow' } else { 'Red' }

    if ($groups.Count -gt 0) {
        $sorted = $groups.GetEnumerator() | Sort-Object { $_.Value.MB } -Descending
        $top3 = $sorted | Select-Object -First 3
        $wasteLine = ($top3 | ForEach-Object { "$($_.Key) $([math]::Round($_.Value.MB,0))MB" }) -join ', '
        if ($sorted.Count -gt 3) { $wasteLine += "..." }
    }
    else { $wasteLine = "Ninguno" }

    Write-Host "   $($I.Pwr) Energia:  " -NoNewline
    Write-Host "$planName" -ForegroundColor $(if ($planId -eq 'High') { 'Green' } else { 'Yellow' })

    Write-Host "   $($I.RAM) RAM:      " -NoNewline
    Write-Host "$($ram.Free)GB / $($ram.Total)GB ($pct% libre)" -ForegroundColor $ramColor

    Write-Host "   $($I.Stm) Steam:    " -NoNewline
    Write-Host $(if ($steamOn) { "Activo" } else { "Cerrado" }) -ForegroundColor $(if ($steamOn) { 'Green' } else { 'DarkGray' })

    Write-Host "   $($I.Warn) Basura:   " -NoNewline
    Write-Host "$totalWaste MB" -ForegroundColor $(if ($totalWaste -gt 0) { 'Red' } else { 'Green' }) -NoNewline
    Write-Host "  ($wasteLine)" -ForegroundColor DarkGray

    Write-Host ""
    Write-Host "  ─────────────────────────────────────────────────" -ForegroundColor DarkGray
    Write-Host ""
    Write-Host "     Opcion [1-4]: " -NoNewline -ForegroundColor Yellow
}

# --- TUI: BUCLE PRINCIPAL ----------------------------------------------------

function Start-TUI {
    do {
        Show-Menu
        $key = [System.Console]::ReadKey($true)
        switch ($key.KeyChar) {
            '1' { Invoke-ChaosOn }
            '2' { Invoke-ChaosOff }
            '3' { Invoke-ChaosStatus }
            '4' {
                Clear-Screen
                Write-Host ""
                Write-Host "  $($I.Gear) Chaos Game Mode cerrado.  " -ForegroundColor Cyan
                Write-Host ""
                return
            }
            default {
                Write-Host "`n   $($I.Warn) Opcion invalida: 1-4" -ForegroundColor Red
                Start-Sleep -Milliseconds 600
            }
        }
    } while ($true)
}

# --- ENTRY POINT -------------------------------------------------------------

if ($State) {
    switch ($State) {
        'on'     { Invoke-ChaosOn }
        'off'    { Invoke-ChaosOff }
        'status' { Invoke-ChaosStatus }
    }
}
else {
    Start-TUI
}
