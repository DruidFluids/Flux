use fluid_core::sensor_data::*;
use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use sysinfo::{Components, CpuRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SensorPoller {
    system: System,
    disks: Disks,
    networks: Networks,
    components: Components,
    nvml: Option<Nvml>,
}

fn shorten_cpu_name(name: &str) -> String {
    let mut n = name.replace("(R)", "").replace("(TM)", "");
    if let Some(idx) = n.find("-Core Processor") {
        if let Some(sp) = n[..idx].rfind(' ') {
            n.truncate(sp);
        }
    }
    n.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl SensorPoller {
    pub fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        let nvml = Nvml::init().ok();
        if nvml.is_some() {
            tracing::info!("NVML initialized - GPU monitoring active");
        } else {
            tracing::warn!("NVML init failed - GPU monitoring unavailable");
        }
        Self {
            system,
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            nvml,
        }
    }

    pub fn poll(&mut self) -> SensorSnapshot {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.disks.refresh(true);
        self.networks.refresh(true);
        self.components.refresh(true);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        SensorSnapshot {
            cpu: self.read_cpu(),
            gpu: self.read_gpu(),
            ram: self.read_ram(),
            disk: self.read_disks(),
            network: self.read_network(),
            timestamp,
        }
    }

    fn read_cpu(&self) -> CpuData {
        let cpus = self.system.cpus();
        let global_usage = self.system.global_cpu_usage();
        let name = cpus.first()
            .map(|c| shorten_cpu_name(c.brand()))
            .unwrap_or_default();
        let clock = cpus.first().map(|c| c.frequency() as f32);

        let temp = self.components.iter()
            .find(|c| {
                let label = c.label().to_lowercase();
                label.contains("cpu") || label.contains("package") || label.contains("tctl")
            })
            .and_then(|c| c.temperature());

        CpuData {
            name,
            usage_percent: global_usage,
            temperature_c: temp,
            clock_mhz: clock,
            core_count: System::physical_core_count().unwrap_or(0) as u32,
            thread_count: cpus.len() as u32,
            per_core_usage: cpus.iter().map(|c| c.cpu_usage()).collect(),
        }
    }

    fn read_gpu(&self) -> GpuData {
        if let Some(nvml) = &self.nvml {
            if let Ok(device) = nvml.device_by_index(0) {
                let name = device.name().unwrap_or_else(|_| "GPU".into())
                    .replace("NVIDIA ", "");
                let usage = device.utilization_rates()
                    .map(|u| u.gpu as f32)
                    .unwrap_or(0.0);
                let temp = device.temperature(TemperatureSensor::Gpu)
                    .ok()
                    .map(|t| t as f32);
                let (vram_used, vram_total) = device.memory_info()
                    .map(|m| (
                        m.used as f32 / (1024.0 * 1024.0),
                        m.total as f32 / (1024.0 * 1024.0),
                    ))
                    .unwrap_or((0.0, 0.0));
                let clock = device.clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics)
                    .ok()
                    .map(|c| c as f32);

                return GpuData {
                    name,
                    usage_percent: usage,
                    temperature_c: temp,
                    vram_used_mb: vram_used,
                    vram_total_mb: vram_total,
                    clock_mhz: clock,
                    ..Default::default()
                };
            }
        }

        let temp = self.components.iter()
            .find(|c| c.label().to_lowercase().contains("gpu"))
            .and_then(|c| c.temperature());

        GpuData {
            name: "GPU".into(),
            temperature_c: temp,
            ..Default::default()
        }
    }

    fn read_ram(&self) -> RamData {
        let used = self.system.used_memory() as f32 / (1024.0 * 1024.0);
        let total = self.system.total_memory() as f32 / (1024.0 * 1024.0);
        RamData {
            used_mb: used,
            total_mb: total,
            usage_percent: if total > 0.0 { (used / total) * 100.0 } else { 0.0 },
        }
    }

    fn read_disks(&self) -> DiskData {
        let mut drives: Vec<DriveInfo> = self.disks.iter().map(|d| {
            let total = d.total_space() as f32 / (1024.0 * 1024.0 * 1024.0);
            let available = d.available_space() as f32 / (1024.0 * 1024.0 * 1024.0);
            let usage = d.usage();
            DriveInfo {
                name: d.name().to_string_lossy().to_string(),
                mount: d.mount_point().to_string_lossy().to_string(),
                total_gb: total,
                used_gb: total - available,
                read_bytes_sec: usage.read_bytes,
                write_bytes_sec: usage.written_bytes,
            }
        }).collect();

        // C: drive first, rest in mount order
        drives.sort_by_key(|d| if d.mount.starts_with("C:") { 0 } else { 1 });

        DiskData { drives }
    }

    fn read_network(&mut self) -> NetworkData {
        let interfaces = self.networks.iter().map(|(name, data)| {
            NetInterface {
                name: name.clone(),
                upload_bytes_sec: data.transmitted(),
                download_bytes_sec: data.received(),
                total_uploaded: data.total_transmitted(),
                total_downloaded: data.total_received(),
            }
        }).collect();

        NetworkData { interfaces }
    }
}
