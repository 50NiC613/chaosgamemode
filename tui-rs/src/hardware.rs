use std::process::Command;

#[derive(Clone)]
pub(crate) struct HardwareState {
    pub(crate) gpu_load_pct: Option<u16>,
    pub(crate) gpu_vram_used_mb: Option<f64>,
    pub(crate) gpu_vram_total_mb: Option<f64>,
    pub(crate) gpu_temp_c: Option<f32>,
    pub(crate) cpu_temp_c: Option<f32>,
    pub(crate) status: String,
}

impl Default for HardwareState {
    fn default() -> Self {
        Self {
            gpu_load_pct: None,
            gpu_vram_used_mb: None,
            gpu_vram_total_mb: None,
            gpu_temp_c: None,
            cpu_temp_c: None,
            status: "hardware sensors pending".to_string(),
        }
    }
}

impl HardwareState {
    pub(crate) fn gpu_vram_used_pct(&self) -> Option<u16> {
        Some(crate::metrics::percent(
            self.gpu_vram_used_mb?,
            self.gpu_vram_total_mb?,
        ))
    }

    fn has_any_reading(&self) -> bool {
        self.gpu_load_pct.is_some()
            || self.gpu_vram_used_mb.is_some()
            || self.gpu_vram_total_mb.is_some()
            || self.gpu_temp_c.is_some()
            || self.cpu_temp_c.is_some()
    }
}

pub(crate) fn read_hardware_state() -> HardwareState {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", HARDWARE_SCRIPT])
        .output();

    match output {
        Ok(output) => parse_hardware_output(&String::from_utf8_lossy(&output.stdout)),
        Err(_) => HardwareState {
            status: "hardware sensors unavailable: powershell".to_string(),
            ..HardwareState::default()
        },
    }
}

fn parse_hardware_output(output: &str) -> HardwareState {
    let mut state = HardwareState::default();
    let mut backend = None;

    for line in output.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "gpu_load_pct" => state.gpu_load_pct = parse_u16(value),
            "gpu_vram_used_mb" => state.gpu_vram_used_mb = parse_f64(value),
            "gpu_vram_total_mb" => state.gpu_vram_total_mb = parse_f64(value),
            "gpu_temp_c" => state.gpu_temp_c = parse_f32(value),
            "cpu_temp_c" => state.cpu_temp_c = parse_f32(value),
            "backend" if !value.is_empty() => backend = Some(value.to_string()),
            _ => {}
        }
    }

    state.status = if state.has_any_reading() {
        format!(
            "hardware sensors: {}",
            backend.unwrap_or_else(|| "windows".to_string())
        )
    } else {
        "hardware sensors unavailable".to_string()
    };
    state
}

fn parse_u16(value: &str) -> Option<u16> {
    parse_f64(value).map(|value| value.clamp(0.0, 100.0).round() as u16)
}

fn parse_f32(value: &str) -> Option<f32> {
    parse_f64(value).map(|value| value as f32)
}

fn parse_f64(value: &str) -> Option<f64> {
    value.trim().replace(',', ".").parse().ok()
}

const HARDWARE_SCRIPT: &str = r#"
$culture = [Globalization.CultureInfo]::InvariantCulture
function EmitMetric($name, $value) {
    if ($null -ne $value -and "$value" -ne "") {
        "$name=$([string]::Format($culture, '{0:0.##}', [double]$value))"
    }
}

$backend = 'windows-counters'
$gpuLoad = $null
$vramUsedMb = $null
$vramTotalMb = $null
$gpuTemp = $null
$cpuTemp = $null

try {
    $samples = (Get-Counter '\GPU Engine(*)\Utilization Percentage' -ErrorAction Stop).CounterSamples |
        Where-Object { $_.InstanceName -match 'engtype_3d|engtype_compute|engtype_graphics' }
    $sum = ($samples | Measure-Object -Property CookedValue -Sum).Sum
    if ($null -ne $sum) {
        $gpuLoad = [Math]::Max(0, [Math]::Min(100, [double]$sum))
    }
} catch {}

try {
    $usage = (Get-Counter '\GPU Adapter Memory(*)\Dedicated Usage' -ErrorAction Stop).CounterSamples
    $sum = ($usage | Measure-Object -Property CookedValue -Sum).Sum
    if ($null -ne $sum) {
        $vramUsedMb = [double]$sum / 1MB
    }
} catch {}

try {
    $limit = (Get-Counter '\GPU Adapter Memory(*)\Dedicated Limit' -ErrorAction Stop).CounterSamples
    $sum = ($limit | Measure-Object -Property CookedValue -Sum).Sum
    if ($null -ne $sum -and [double]$sum -gt 0) {
        $vramTotalMb = [double]$sum / 1MB
    }
} catch {}

foreach ($namespace in @('root\LibreHardwareMonitor', 'root\OpenHardwareMonitor')) {
    try {
        $sensors = Get-CimInstance -Namespace $namespace -ClassName Sensor -ErrorAction Stop
        if ($sensors) {
            $backend = $namespace
            if ($null -eq $gpuTemp) {
                $gpuSensor = $sensors |
                    Where-Object { $_.SensorType -eq 'Temperature' -and ($_.Name -match 'GPU' -or $_.Identifier -match '/gpu') } |
                    Sort-Object -Property Value -Descending |
                    Select-Object -First 1
                if ($gpuSensor) { $gpuTemp = [double]$gpuSensor.Value }
            }
            if ($null -eq $cpuTemp) {
                $cpuSensor = $sensors |
                    Where-Object { $_.SensorType -eq 'Temperature' -and ($_.Name -match 'CPU|Tctl|Tdie|Package' -or $_.Identifier -match '/cpu') } |
                    Sort-Object -Property Value -Descending |
                    Select-Object -First 1
                if ($cpuSensor) { $cpuTemp = [double]$cpuSensor.Value }
            }
        }
    } catch {}
}

EmitMetric 'gpu_load_pct' $gpuLoad
EmitMetric 'gpu_vram_used_mb' $vramUsedMb
EmitMetric 'gpu_vram_total_mb' $vramTotalMb
EmitMetric 'gpu_temp_c' $gpuTemp
EmitMetric 'cpu_temp_c' $cpuTemp
"backend=$backend"
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hardware_output_should_read_gpu_metrics() {
        let state = parse_hardware_output(
            "gpu_load_pct=42\ngpu_vram_used_mb=1024.5\ngpu_vram_total_mb=4096\ngpu_temp_c=71\nbackend=test\n",
        );

        assert_eq!(state.gpu_load_pct, Some(42));
    }

    #[test]
    fn gpu_vram_used_pct_should_calculate_percentage_when_available() {
        let state = HardwareState {
            gpu_vram_used_mb: Some(1024.0),
            gpu_vram_total_mb: Some(4096.0),
            ..HardwareState::default()
        };

        assert_eq!(state.gpu_vram_used_pct(), Some(25));
    }

    #[test]
    fn parse_hardware_output_should_report_unavailable_when_empty() {
        let state = parse_hardware_output("");

        assert_eq!(state.status, "hardware sensors unavailable");
    }
}
