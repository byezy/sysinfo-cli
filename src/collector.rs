use sysinfo::{
    Components, CpuRefreshKind, Disks, MemoryRefreshKind, Networks, 
    ProcessRefreshKind, RefreshKind, System, ProcessesToUpdate
};
use std::thread;
use crate::args::{Commands, SortBy};
use crate::models::*;

pub fn init_system(command: &Option<Commands>) -> System {
    let mut sys = match command {
        Some(Commands::System) => System::new_with_specifics(RefreshKind::nothing()),
        Some(Commands::Cpu) => {
            let mut s = System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()));
            thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
            s.refresh_cpu_usage();
            s
        }
        Some(Commands::Memory) => System::new_with_specifics(RefreshKind::nothing().with_memory(MemoryRefreshKind::nothing().with_ram().with_swap())),
        Some(Commands::Processes { .. }) => {
            let mut s = System::new_with_specifics(
                RefreshKind::nothing()
                    .with_processes(ProcessRefreshKind::nothing().with_cpu().with_memory())
                    .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
            );
            thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
            s.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                ProcessRefreshKind::nothing().with_cpu().with_memory()
            );
            s
        }
        _ => System::new_with_specifics(
                RefreshKind::nothing()
                    .with_memory(MemoryRefreshKind::nothing().with_ram())
                    .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
            ),
    };

    if command.is_none() {
        thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_cpu_usage();
    }
    
    sys
}

pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        name: System::name(),
        kernel_version: System::kernel_version(),
        os_version: System::os_version(),
        host_name: System::host_name(),
    }
}

pub fn get_cpu_info(sys: &System) -> CpuInfo {
    CpuInfo {
        nb_cpus: sys.cpus().len(),
        cpus: sys.cpus().iter().enumerate().map(|(i, cpu)| SingleCpuInfo {
            id: i,
            usage: cpu.cpu_usage(),
            vendor: cpu.vendor_id().to_string(),
            brand: cpu.brand().to_string(),
        }).collect(),
        total_usage: sys.global_cpu_usage(),
    }
}

pub fn get_memory_info(sys: &System) -> MemoryInfo {
    MemoryInfo {
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        total_swap: sys.total_swap(),
        used_swap: sys.used_swap(),
    }
}

pub fn get_disks_info() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    disks.iter().map(|disk| DiskInfo {
        name: disk.name().to_string_lossy().into_owned(),
        kind: disk.kind().to_string(),
        file_system: disk.file_system().to_string_lossy().into_owned(),
        available_space: disk.available_space(),
        total_space: disk.total_space(),
    }).collect()
}

pub fn get_network_info() -> Vec<NetworkInfo> {
    let networks = Networks::new_with_refreshed_list();
    networks.iter().map(|(name, data)| NetworkInfo {
        interface: name.clone(),
        received: data.total_received(),
        transmitted: data.total_transmitted(),
    }).collect()
}

pub fn get_components_info() -> Vec<ComponentInfo> {
    let components = Components::new_with_refreshed_list();
    components.iter().map(|c| ComponentInfo {
        label: c.label().to_string(),
        temperature: c.temperature(),
        max: c.max(),
    }).collect()
}

pub fn get_processes_info(sys: &System, filter: &Option<String>, limit: Option<usize>, sort: SortBy) -> Vec<ProcessInfo> {
    let mut processes: Vec<ProcessInfo> = sys.processes().values().filter(|p| {
        if let Some(f) = filter {
            p.name().to_string_lossy().contains(f)
        } else {
            true
        }
    }).map(|p| ProcessInfo {
        pid: p.pid().to_string(),
        name: p.name().to_string_lossy().into_owned(),
        cpu_usage: p.cpu_usage(),
        memory: p.memory(),
    }).collect();

    match sort {
        SortBy::Cpu => processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap()),
        SortBy::Memory => processes.sort_by(|a, b| b.memory.cmp(&a.memory)),
        SortBy::Pid => processes.sort_by(|a, b| a.pid.cmp(&b.pid)),
        SortBy::Name => processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
    }

    if let Some(l) = limit {
        processes.truncate(l);
    }

    processes
}

