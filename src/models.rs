use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct SystemInfo {
    pub name: Option<String>,
    pub kernel_version: Option<String>,
    pub os_version: Option<String>,
    pub host_name: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct CpuInfo {
    pub nb_cpus: usize,
    pub cpus: Vec<SingleCpuInfo>,
    pub total_usage: f32,
}

#[derive(Serialize, Debug)]
pub struct SingleCpuInfo {
    pub id: usize,
    pub usage: f32,
    pub vendor: String,
    pub brand: String,
}

#[derive(Serialize, Debug)]
pub struct MemoryInfo {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}

#[derive(Serialize, Debug)]
pub struct DiskInfo {
    pub name: String,
    pub kind: String,
    pub file_system: String,
    pub available_space: u64,
    pub total_space: u64,
}

#[derive(Serialize, Debug)]
pub struct NetworkInfo {
    pub interface: String,
    pub received: u64,
    pub transmitted: u64,
}

#[derive(Serialize, Debug)]
pub struct ComponentInfo {
    pub label: String,
    pub temperature: Option<f32>,
    pub max: Option<f32>,
}

#[derive(Serialize, Debug)]
pub struct ProcessInfo {
    pub pid: String,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
}

