use std::collections::HashMap;
use std::process::Command;

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::config::BoostProfile;
use crate::hardware::{HardwareState, read_hardware_state};
use crate::i18n::Language;
use crate::metrics::percent;

const HIGH_PERF_GUID: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
const BALANCED_GUID: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";

#[derive(Clone)]
pub(crate) struct ProcessGroup {
    pub(crate) count: usize,
    pub(crate) memory_mb: f64,
    pub(crate) exe_path: Option<String>,
}

#[derive(Clone)]
pub(crate) struct SystemState {
    pub(crate) cpu_usage: f32,
    pub(crate) cpu_cores: usize,
    pub(crate) ram_total_gb: f64,
    pub(crate) ram_used_gb: f64,
    pub(crate) ram_free_gb: f64,
    pub(crate) power_plan: String,
    pub(crate) explorer_on: bool,
    pub(crate) steam_on: bool,
    pub(crate) steam_mb: f64,
    pub(crate) services_running: usize,
    pub(crate) hardware: HardwareState,
    pub(crate) observed_processes: HashMap<String, ProcessGroup>,
    pub(crate) hidden_processes: HashMap<String, ProcessGroup>,
    pub(crate) processes: HashMap<String, ProcessGroup>,
    pub(crate) total_waste_mb: f64,
}

struct ProcessScan {
    observed_processes: HashMap<String, ProcessGroup>,
    hidden_processes: HashMap<String, ProcessGroup>,
    target_processes: HashMap<String, ProcessGroup>,
    total_waste_mb: f64,
}

pub(crate) struct ActionReport {
    status: ActionStatus,
    message: String,
}

enum ActionStatus {
    Info,
    Success,
    Warning,
}

impl ActionReport {
    fn info(message: impl Into<String>) -> Self {
        Self {
            status: ActionStatus::Info,
            message: message.into(),
        }
    }

    fn success(message: impl Into<String>) -> Self {
        Self {
            status: ActionStatus::Success,
            message: message.into(),
        }
    }

    fn warning(message: impl Into<String>) -> Self {
        Self {
            status: ActionStatus::Warning,
            message: message.into(),
        }
    }

    fn line(&self) -> String {
        let marker = match self.status {
            ActionStatus::Info => "\u{f05a}",
            ActionStatus::Success => "\u{f00c}",
            ActionStatus::Warning => "\u{f071}",
        };
        format!("  {marker} {}", self.message)
    }
}

pub(crate) fn action_lines(reports: &[ActionReport]) -> Vec<String> {
    reports.iter().map(ActionReport::line).collect()
}

impl SystemState {
    pub(crate) fn ram_used_pct(&self) -> u16 {
        percent(self.ram_used_gb, self.ram_total_gb)
    }

    pub(crate) fn ram_free_pct(&self) -> u16 {
        percent(self.ram_free_gb, self.ram_total_gb)
    }

    #[cfg(test)]
    pub(crate) fn empty_for_test() -> Self {
        Self {
            cpu_usage: 0.0,
            cpu_cores: 0,
            ram_total_gb: 0.0,
            ram_used_gb: 0.0,
            ram_free_gb: 0.0,
            power_plan: String::new(),
            explorer_on: false,
            steam_on: false,
            steam_mb: 0.0,
            services_running: 0,
            hardware: HardwareState::default(),
            observed_processes: HashMap::new(),
            hidden_processes: HashMap::new(),
            processes: HashMap::new(),
            total_waste_mb: 0.0,
        }
    }
}

fn run_powershell(script: &str) -> String {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output();
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => String::new(),
    }
}

fn get_power_plan_name() -> String {
    let out = run_powershell("powercfg /getactivescheme");
    if out.contains(HIGH_PERF_GUID) {
        "Alto Rendimiento".to_string()
    } else if out.contains(BALANCED_GUID) {
        "Balanceado".to_string()
    } else {
        "Personalizado".to_string()
    }
}

fn set_power_plan(guid: &str) -> bool {
    let output = Command::new("powercfg").args(["/setactive", guid]).output();
    output.is_ok_and(|o| o.status.success())
}

fn is_explorer_running() -> bool {
    let out = run_powershell(
        "if (Get-Process -Name 'explorer' -ErrorAction SilentlyContinue) { '1' } else { '0' }",
    );
    out == "1"
}

fn get_steam_info() -> (bool, f64) {
    let out = run_powershell(
        "$p = Get-Process -Name 'steam' -ErrorAction SilentlyContinue; \
         if ($p) { \"1,$($p.WorkingSet64)\" } else { '0,0' }",
    );
    let parts: Vec<&str> = out.split(',').collect();
    if parts.len() == 2 && parts[0] == "1" {
        let bytes: f64 = parts[1].parse().unwrap_or(0.0);
        (true, bytes / 1_048_576.0)
    } else {
        (false, 0.0)
    }
}

fn get_services_running(services: &[String]) -> usize {
    if services.is_empty() {
        return 0;
    }

    let names = powershell_array(services);
    let script = format!(
        "@({names}) | \
         ForEach-Object {{ if ((Get-Service -Name $_ -ErrorAction SilentlyContinue).Status -eq 'Running') {{ $_ }} }} | \
         Measure-Object | Select-Object -ExpandProperty Count"
    );
    let out = run_powershell(&script);
    out.trim().parse().unwrap_or(0)
}

fn kill_processes(profile: &BoostProfile, language: Language) -> Vec<ActionReport> {
    let mut log = Vec::new();
    let processes = filtered_process_patterns(profile);
    if processes.is_empty() {
        log.push(ActionReport::info(language.system_info_no_processes()));
        return log;
    }

    let patterns = powershell_array(&processes);
    let script = format!(
        "@({patterns}) | ForEach-Object {{ \
         $p = Get-Process -Name $_ -ErrorAction SilentlyContinue; \
         if ($p) {{ \
         $mb = [Math]::Round(($p.WorkingSet64 / 1MB), 1); \
         Stop-Process -Name $_ -Force -ErrorAction SilentlyContinue; \
         Write-Output \"$($_): $mb MB\" \
         }} \
         }}",
    );
    let out = run_powershell(&script);
    for line in out.lines() {
        if !line.is_empty() {
            log.push(ActionReport::success(language.system_process_closed(line)));
        }
    }
    if log.is_empty() {
        log.push(ActionReport::info(language.system_no_heavy_process()));
    }
    log
}

fn stop_services(services: &[String], language: Language) -> Vec<ActionReport> {
    let mut log = Vec::new();
    if services.is_empty() {
        log.push(ActionReport::info(language.system_no_services()));
        return log;
    }

    let names = powershell_array(services);
    let script = format!(
        "@({names}) | ForEach-Object {{ \
         $s = Get-Service -Name $_ -ErrorAction SilentlyContinue; \
         if ($s -and $s.Status -eq 'Running') {{ \
         Stop-Service -Name $_ -Force -ErrorAction SilentlyContinue; \
         Write-Output $_ \
         }} \
         }}",
    );
    let out = run_powershell(&script);
    for line in out.lines() {
        if !line.is_empty() {
            log.push(ActionReport::success(language.system_service_stopped(line)));
        }
    }
    if log.is_empty() {
        log.push(ActionReport::info(language.system_services_optimized()));
    }
    log
}

fn priority_steam(language: Language) -> Vec<ActionReport> {
    let mut log = Vec::new();
    let script = run_powershell(
        "$p = Get-Process -Name 'steam' -ErrorAction SilentlyContinue; \
         if (-not $p) { \
         $found = $false; \
         @('C:\\Program Files (x86)\\Steam\\steam.exe','C:\\Program Files\\Steam\\steam.exe') | \
         ForEach-Object {{ if (Test-Path $_) {{ Start-Process $_; $found = $true }} }}; \
         if (-not $found) {{ Write-Output 'NO_STEAM' }} \
         } else { \
         Write-Output 'OK' \
         }",
    );
    if script == "NO_STEAM" {
        log.push(ActionReport::warning(language.steam_not_found_manual()));
    } else if script == "OK" {
        log.push(ActionReport::success(language.steam_already_active()));
    } else {
        log.push(ActionReport::success(language.steam_opened()));
    }
    log
}

fn kill_explorer(language: Language) -> Vec<ActionReport> {
    let mut log = Vec::new();
    let script = run_powershell(
        "$p = Get-Process -Name 'explorer' -ErrorAction SilentlyContinue; \
         if ($p) { Stop-Process -Name 'explorer' -Force; Write-Output 'KILLED' }",
    );
    if script == "KILLED" {
        log.push(ActionReport::success(language.explorer_stopped()));
    }
    log
}

fn start_explorer(language: Language) -> Vec<ActionReport> {
    let mut log = Vec::new();
    let script = run_powershell(
        "if (-not (Get-Process -Name 'explorer' -ErrorAction SilentlyContinue)) { \
         Start-Process 'explorer.exe'; Write-Output 'STARTED' }",
    );
    if script == "STARTED" {
        log.push(ActionReport::success(language.explorer_started()));
    }
    log
}

pub(crate) fn activate_chaos_mode(profile: &BoostProfile, language: Language) -> Vec<ActionReport> {
    let mut log = Vec::new();
    log.push(ActionReport::info(format!(
        "{}: {}",
        language.active_profile_line(),
        profile.name
    )));
    if profile.set_high_performance {
        log.push(ActionReport::info(format!(
            "{}: {}",
            language.power_plan_line(),
            language.high_performance_plan()
        )));
        set_power_plan(HIGH_PERF_GUID);
    } else {
        log.push(ActionReport::info(format!(
            "{}: {}",
            language.power_plan_line(),
            language.no_changes()
        )));
    }

    log.push(ActionReport::info(language.killing_background_processes()));
    log.extend(kill_processes(profile, language));

    log.push(ActionReport::info(language.stopping_services()));
    log.extend(stop_services(&profile.services, language));

    if profile.prioritize_steam {
        log.push(ActionReport::info("Steam"));
        log.extend(priority_steam(language));
    }

    if profile.kill_explorer {
        log.push(ActionReport::info(language.freeing_system_resources()));
        log.extend(kill_explorer(language));
    } else {
        log.push(ActionReport::info(language.explorer_kept_report()));
    }

    log.push(ActionReport::success(language.overdrive_activated()));
    log
}

pub(crate) fn restore_system(profile: &BoostProfile, language: Language) -> Vec<ActionReport> {
    let mut log = Vec::new();
    log.push(ActionReport::info(format!(
        "{}: {}",
        language.active_profile_line(),
        profile.name
    )));
    log.push(ActionReport::info(language.restoring_windows_shell()));
    log.extend(start_explorer(language));

    log.push(ActionReport::info(language.restoring_services()));
    if profile.services.is_empty() {
        log.push(ActionReport::info(language.system_no_services()));
    } else {
        let names = powershell_array(&profile.services);
        let script = run_powershell(&format!(
            "@({names}) | \
         ForEach-Object {{ \
         $s = Get-Service -Name $_ -ErrorAction SilentlyContinue; \
         if ($s -and $s.Status -ne 'Running') {{ \
         Start-Service -Name $_ -ErrorAction SilentlyContinue; \
         Write-Output $_ \
         }} \
         }}"
        ));
        for line in script.lines() {
            if !line.is_empty() {
                log.push(ActionReport::success(language.system_service_started(line)));
            }
        }
    }

    log.push(ActionReport::info(format!(
        "{}: {}",
        language.power_plan_line(),
        language.balanced_plan()
    )));
    set_power_plan(BALANCED_GUID);

    log.push(ActionReport::success(language.system_restored()));
    log.push(ActionReport::warning(language.closed_apps_not_reopened()));
    log
}

pub(crate) fn refresh_system_state(
    sys: &mut System,
    previous: Option<&SystemState>,
    refresh_processes: bool,
    refresh_platform: bool,
    profile: &BoostProfile,
) -> SystemState {
    sys.refresh_memory();
    sys.refresh_cpu_usage();

    let total_bytes = sys.total_memory();
    let free_bytes = sys.free_memory();
    let used_bytes = total_bytes.saturating_sub(free_bytes);
    let ram_total_gb = total_bytes as f64 / 1_073_741_824.0;
    let ram_free_gb = free_bytes as f64 / 1_073_741_824.0;
    let ram_used_gb = used_bytes as f64 / 1_073_741_824.0;

    let process_scan = match (refresh_processes, previous) {
        (false, Some(state)) => ProcessScan {
            observed_processes: state.observed_processes.clone(),
            hidden_processes: state.hidden_processes.clone(),
            target_processes: state.processes.clone(),
            total_waste_mb: state.total_waste_mb,
        },
        _ => scan_target_processes(sys, profile),
    };

    let (power_plan, explorer_on, steam_on, steam_mb, services_running, hardware) =
        match (refresh_platform, previous) {
            (false, Some(state)) => (
                state.power_plan.clone(),
                state.explorer_on,
                state.steam_on,
                state.steam_mb,
                state.services_running,
                state.hardware.clone(),
            ),
            _ => {
                let power_plan = get_power_plan_name();
                let explorer_on = is_explorer_running();
                let (steam_on, steam_mb) = get_steam_info();
                let services_running = get_services_running(&profile.services);
                let hardware = read_hardware_state();
                (
                    power_plan,
                    explorer_on,
                    steam_on,
                    steam_mb,
                    services_running,
                    hardware,
                )
            }
        };

    SystemState {
        cpu_usage: sys.global_cpu_usage(),
        cpu_cores: sys.cpus().len(),
        ram_total_gb,
        ram_used_gb,
        ram_free_gb,
        power_plan,
        explorer_on,
        steam_on,
        steam_mb,
        services_running,
        hardware,
        observed_processes: process_scan.observed_processes,
        hidden_processes: process_scan.hidden_processes,
        processes: process_scan.target_processes,
        total_waste_mb: process_scan.total_waste_mb,
    }
}

fn scan_target_processes(sys: &mut System, profile: &BoostProfile) -> ProcessScan {
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::new()
            .with_memory()
            .with_exe(UpdateKind::OnlyIfNotSet),
    );

    let mut observed_groups: HashMap<String, ProcessGroup> = HashMap::new();
    let mut hidden_groups: HashMap<String, ProcessGroup> = HashMap::new();
    let mut target_groups: HashMap<String, ProcessGroup> = HashMap::new();
    let mut total_waste_mb = 0.0_f64;

    for process in sys.processes().values() {
        let name = process.name().to_string_lossy();
        let name_lower = name.to_lowercase();
        let mem_mb = process.memory() as f64 / 1_048_576.0;
        let exe_path = process
            .exe()
            .map(|path| path.display().to_string())
            .filter(|path| !path.trim().is_empty());

        let groups = if profile.is_hidden_process(&name_lower) {
            &mut hidden_groups
        } else {
            &mut observed_groups
        };
        record_process_sample(groups, &name, mem_mb, exe_path.as_deref());

        if profile.is_target_process(&name_lower) {
            record_process_sample(&mut target_groups, &name, mem_mb, exe_path.as_deref());
            total_waste_mb += mem_mb;
        }
    }

    ProcessScan {
        observed_processes: observed_groups,
        hidden_processes: hidden_groups,
        target_processes: target_groups,
        total_waste_mb,
    }
}

fn record_process_sample(
    groups: &mut HashMap<String, ProcessGroup>,
    name: &str,
    memory_mb: f64,
    exe_path: Option<&str>,
) {
    let entry = groups.entry(name.to_string()).or_insert(ProcessGroup {
        count: 0,
        memory_mb: 0.0,
        exe_path: None,
    });
    entry.count += 1;
    entry.memory_mb += memory_mb;
    if entry.exe_path.is_none() {
        entry.exe_path = exe_path.map(ToOwned::to_owned);
    }
}

fn filtered_process_patterns(profile: &BoostProfile) -> Vec<String> {
    profile
        .processes
        .iter()
        .filter(|pattern| {
            let process_name = pattern.to_ascii_lowercase();
            !profile.is_protected_process(&process_name)
                && !profile.is_hidden_process(&process_name)
        })
        .cloned()
        .collect()
}

fn powershell_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("'{}'", value.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protected_process_should_not_be_targeted() {
        let profile = BoostProfile {
            name: "test",
            processes: vec!["steelseriesgg".to_string(), "chrome".to_string()],
            protected_processes: vec!["steelseries".to_string()],
            hidden_processes: Vec::new(),
            services: Vec::new(),
            set_high_performance: false,
            prioritize_steam: false,
            kill_explorer: false,
        };

        assert!(!profile.is_target_process("steelseriesgg"));
        assert!(profile.is_target_process("chrome"));
    }

    #[test]
    fn filtered_process_patterns_should_drop_protected_and_hidden_entries() {
        let profile = BoostProfile {
            name: "test",
            processes: vec![
                "steelseriesgg".to_string(),
                "chrome".to_string(),
                "searchhost".to_string(),
            ],
            protected_processes: vec!["steelseries".to_string()],
            hidden_processes: vec!["searchhost".to_string()],
            services: Vec::new(),
            set_high_performance: false,
            prioritize_steam: false,
            kill_explorer: false,
        };

        assert_eq!(filtered_process_patterns(&profile), vec!["chrome"]);
    }
}
