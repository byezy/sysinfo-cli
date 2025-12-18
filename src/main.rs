use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use comfy_table::Table;
use serde::Serialize;
use sysinfo::{
    Components, CpuRefreshKind, Disks, MemoryRefreshKind, Networks, 
    ProcessRefreshKind, RefreshKind, System, ProcessesToUpdate
};
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Write;

#[derive(Parser)]
#[command(name = "sysinfo-cli")]
#[command(about = "A useful CLI wrapper around the sysinfo crate", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Output in JSON format
    #[arg(short, long, global = true)]
    json: bool,

    /// Refresh interval in seconds for continuous monitoring
    #[arg(short, long, global = true)]
    watch: Option<u64>,

    /// Save output to a file
    #[arg(short, long, global = true)]
    output: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show general system information
    System,
    /// Show CPU information
    Cpu,
    /// Show memory and swap information
    Memory,
    /// Show disk information
    Disks,
    /// Show network information
    Network,
    /// Show components (temperature, etc.)
    Components,
    /// Show running processes
    Processes {
        /// Filter processes by name
        #[arg(short, long)]
        filter: Option<String>,
        /// Number of processes to show (default: all)
        #[arg(short, long)]
        limit: Option<usize>,
        /// Sort by a specific criteria
        #[arg(short, long, value_enum, default_value_t = SortBy::Cpu)]
        sort: SortBy,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum SortBy {
    Cpu,
    Memory,
    Pid,
    Name,
}

#[derive(Serialize)]
struct SystemInfo {
    name: Option<String>,
    kernel_version: Option<String>,
    os_version: Option<String>,
    host_name: Option<String>,
}

#[derive(Serialize)]
struct CpuInfo {
    nb_cpus: usize,
    cpus: Vec<SingleCpuInfo>,
    total_usage: f32,
}

#[derive(Serialize)]
struct SingleCpuInfo {
    id: usize,
    usage: f32,
    vendor: String,
    brand: String,
}

#[derive(Serialize)]
struct MemoryInfo {
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64,
}

#[derive(Serialize)]
struct DiskInfo {
    name: String,
    kind: String,
    file_system: String,
    available_space: u64,
    total_space: u64,
}

#[derive(Serialize)]
struct NetworkInfo {
    interface: String,
    received: u64,
    transmitted: u64,
}

#[derive(Serialize)]
struct ComponentInfo {
    label: String,
    temperature: Option<f32>,
    max: Option<f32>,
}

#[derive(Serialize)]
struct ProcessInfo {
    pid: String,
    name: String,
    cpu_usage: f32,
    memory: u64,
}

fn main() {
    let cli = Cli::parse();
    
    loop {
        let sys = init_system(&cli.command);
        let mut output_str = String::new();
        
        match &cli.command {
            Some(Commands::System) => {
                let info = get_system_info();
                if cli.json {
                    output_str.push_str(&serde_json::to_string_pretty(&info).unwrap());
                } else {
                    output_str.push_str(&format_system_info(&info));
                }
            }
            Some(Commands::Cpu) => {
                let info = get_cpu_info(&sys);
                if cli.json {
                    output_str.push_str(&serde_json::to_string_pretty(&info).unwrap());
                } else {
                    output_str.push_str(&format_cpu_info(&info));
                }
            }
            Some(Commands::Memory) => {
                let info = get_memory_info(&sys);
                if cli.json {
                    output_str.push_str(&serde_json::to_string_pretty(&info).unwrap());
                } else {
                    output_str.push_str(&format_memory_info(&info));
                }
            }
            Some(Commands::Disks) => {
                let info = get_disks_info();
                if cli.json {
                    output_str.push_str(&serde_json::to_string_pretty(&info).unwrap());
                } else {
                    output_str.push_str(&format_disks_info(&info));
                }
            }
            Some(Commands::Network) => {
                let info = get_network_info();
                if cli.json {
                    output_str.push_str(&serde_json::to_string_pretty(&info).unwrap());
                } else {
                    output_str.push_str(&format_network_info(&info));
                }
            }
            Some(Commands::Components) => {
                let info = get_components_info();
                if cli.json {
                    output_str.push_str(&serde_json::to_string_pretty(&info).unwrap());
                } else {
                    output_str.push_str(&format_components_info(&info));
                }
            }
            Some(Commands::Processes { filter, limit, sort }) => {
                let info = get_processes_info(&sys, filter, *limit, *sort);
                if cli.json {
                    output_str.push_str(&serde_json::to_string_pretty(&info).unwrap());
                } else {
                    output_str.push_str(&format_processes_info(&info));
                }
            }
            None => {
                if cli.json {
                    let summary = serde_json::json!({
                        "system": get_system_info(),
                        "memory": get_memory_info(&sys),
                        "cpu_total_usage": sys.global_cpu_usage(),
                        "nb_cpus": sys.cpus().len(),
                    });
                    output_str.push_str(&serde_json::to_string_pretty(&summary).unwrap());
                } else {
                    let mut s = String::new();
                    s.push_str(&format!("{}\n", "--- System Summary ---".bright_cyan().bold()));
                    s.push_str(&format_system_info(&get_system_info()));
                    s.push_str(&format!("\n{}\n", "--- Memory Summary ---".bright_cyan().bold()));
                    let mem = get_memory_info(&sys);
                    s.push_str(&format!("{:<25} {}\n", "Total memory:".yellow(), format_bytes(mem.total_memory)));
                    s.push_str(&format!("{:<25} {}\n", "Used memory:".yellow(), format_bytes(mem.used_memory)));
                    s.push_str(&format!("\n{}\n", "--- CPU Summary ---".bright_cyan().bold()));
                    s.push_str(&format!("{:<25} {}\n", "NB CPUs:".yellow(), sys.cpus().len()));
                    s.push_str(&format!("{:<25} {:.1}%\n", "Total CPU usage:".yellow(), sys.global_cpu_usage()));
                    output_str.push_str(&s);
                }
            }
        }

        if let Some(path) = &cli.output {
            if let Ok(mut file) = File::create(path) {
                if let Err(e) = write!(file, "{}", output_str) {
                    eprintln!("Error writing to file: {}", e);
                }
            } else {
                eprintln!("Error creating file: {}", path);
            }
        } else {
            println!("{}", output_str);
        }

        if let Some(interval) = cli.watch {
            thread::sleep(Duration::from_secs(interval));
            if !cli.json && cli.output.is_none() {
                // Clear screen for watch mode if not in JSON or File mode
                print!("\x1B[2J\x1B[1;1H");
            }
        } else {
            break;
        }
    }
}

fn init_system(command: &Option<Commands>) -> System {
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

    if let None = command {
        thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_cpu_usage();
    }
    
    sys
}

fn get_system_info() -> SystemInfo {
    SystemInfo {
        name: System::name(),
        kernel_version: System::kernel_version(),
        os_version: System::os_version(),
        host_name: System::host_name(),
    }
}

fn format_system_info(info: &SystemInfo) -> String {
    let mut s = String::new();
    s.push_str(&format!("{:<25} {:?}\n", "System name:".yellow(), info.name.as_deref().unwrap_or_default()));
    s.push_str(&format!("{:<25} {:?}\n", "Kernel version:".yellow(), info.kernel_version.as_deref().unwrap_or_default()));
    s.push_str(&format!("{:<25} {:?}\n", "OS version:".yellow(), info.os_version.as_deref().unwrap_or_default()));
    s.push_str(&format!("{:<25} {:?}\n", "Host name:".yellow(), info.host_name.as_deref().unwrap_or_default()));
    s
}

fn get_cpu_info(sys: &System) -> CpuInfo {
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

fn format_cpu_info(info: &CpuInfo) -> String {
    let mut s = String::new();
    s.push_str(&format!("{}\n", "=> CPUs:".bright_green().bold()));
    s.push_str(&format!("{:<25} {}\n", "Total CPUs:".yellow(), info.nb_cpus));
    s.push_str(&format!("{:<25} {:.1}%\n", "Global usage:".yellow(), info.total_usage));
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Usage %", "Vendor", "Brand"]);
    for cpu in &info.cpus {
        table.add_row(vec![
            cpu.id.to_string(),
            format!("{:.1}", cpu.usage),
            cpu.vendor.clone(),
            cpu.brand.clone(),
        ]);
    }
    s.push_str(&format!("{}\n", table));
    s
}

fn get_memory_info(sys: &System) -> MemoryInfo {
    MemoryInfo {
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        total_swap: sys.total_swap(),
        used_swap: sys.used_swap(),
    }
}

fn format_memory_info(info: &MemoryInfo) -> String {
    let mut s = String::new();
    s.push_str(&format!("{:<25} {}\n", "Total memory:".yellow(), format_bytes(info.total_memory)));
    s.push_str(&format!("{:<25} {}\n", "Used memory:".yellow(), format_bytes(info.used_memory)));
    s.push_str(&format!("{:<25} {}\n", "Total swap:".yellow(), format_bytes(info.total_swap)));
    s.push_str(&format!("{:<25} {}\n", "Used swap:".yellow(), format_bytes(info.used_swap)));
    s
}

fn get_disks_info() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    disks.iter().map(|disk| DiskInfo {
        name: disk.name().to_string_lossy().into_owned(),
        kind: disk.kind().to_string(),
        file_system: disk.file_system().to_string_lossy().into_owned(),
        available_space: disk.available_space(),
        total_space: disk.total_space(),
    }).collect()
}

fn format_disks_info(info: &[DiskInfo]) -> String {
    let mut s = String::new();
    s.push_str(&format!("{}\n", "=> Disks:".bright_green().bold()));
    let mut table = Table::new();
    table.set_header(vec!["Name", "Kind", "FS", "Available", "Total"]);
    for disk in info {
        table.add_row(vec![
            disk.name.cyan().to_string(),
            disk.kind.blue().to_string(),
            disk.file_system.yellow().to_string(),
            format_bytes(disk.available_space),
            format_bytes(disk.total_space),
        ]);
    }
    s.push_str(&format!("{}\n", table));
    s
}

fn get_network_info() -> Vec<NetworkInfo> {
    let networks = Networks::new_with_refreshed_list();
    networks.iter().map(|(name, data)| NetworkInfo {
        interface: name.clone(),
        received: data.total_received(),
        transmitted: data.total_transmitted(),
    }).collect()
}

fn format_network_info(info: &[NetworkInfo]) -> String {
    let mut s = String::new();
    s.push_str(&format!("{}\n", "=> Networks:".bright_green().bold()));
    let mut table = Table::new();
    table.set_header(vec!["Interface", "Received", "Transmitted"]);
    for net in info {
        table.add_row(vec![
            net.interface.cyan().to_string(),
            format_bytes(net.received).yellow().to_string(),
            format_bytes(net.transmitted).yellow().to_string(),
        ]);
    }
    s.push_str(&format!("{}\n", table));
    s
}

fn get_components_info() -> Vec<ComponentInfo> {
    let components = Components::new_with_refreshed_list();
    components.iter().map(|c| ComponentInfo {
        label: c.label().to_string(),
        temperature: c.temperature(),
        max: c.max(),
    }).collect()
}

fn format_components_info(info: &[ComponentInfo]) -> String {
    let mut s = String::new();
    s.push_str(&format!("{}\n", "=> Components:".bright_green().bold()));
    let mut table = Table::new();
    table.set_header(vec!["Label", "Temp", "Max"]);
    for c in info {
        table.add_row(vec![
            c.label.cyan().to_string(),
            format!("{}°C", c.temperature.map(|t| format!("{:.1}", t)).unwrap_or_else(|| "N/A".to_string())),
            format!("{}°C", c.max.map(|t| format!("{:.1}", t)).unwrap_or_else(|| "N/A".to_string())),
        ]);
    }
    s.push_str(&format!("{}\n", table));
    s
}

fn get_processes_info(sys: &System, filter: &Option<String>, limit: Option<usize>, sort: SortBy) -> Vec<ProcessInfo> {
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

fn format_processes_info(info: &[ProcessInfo]) -> String {
    let mut s = String::new();
    s.push_str(&format!("{}\n", "=> Processes:".bright_green().bold()));
    let mut table = Table::new();
    table.set_header(vec!["PID", "Name", "CPU %", "Memory"]);
    for p in info {
        let name = if p.name.len() > 30 { format!("{}...", &p.name[..27]) } else { p.name.clone() };
        table.add_row(vec![
            p.pid.cyan().to_string(),
            name,
            format!("{:>5.1}", p.cpu_usage),
            format_bytes(p.memory),
        ]);
    }
    s.push_str(&format!("{}\n", table));
    s
}

fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let i = (bytes as f64).log(1024.0).floor() as usize;
    let i = i.min(units.len() - 1);
    let value = bytes as f64 / 1024.0f64.powi(i as i32);
    format!("{:.2} {}", value, units[i])
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1), "1.00 B");
        assert_eq!(format_bytes(1023), "1023.00 B");
        assert_eq!(format_bytes(1024), "1.00 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GiB");
    }

    #[test]
    fn test_cli_parsing_default() {
        let args = vec!["sysinfo-cli"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.command.is_none());
        assert!(!cli.json);
        assert_eq!(cli.watch, None);
    }

    #[test]
    fn test_cli_parsing_watch_json() {
        let args = vec!["sysinfo-cli", "--watch", "2", "--json", "system"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.watch, Some(2));
        assert!(cli.json);
        match cli.command {
            Some(Commands::System) => (),
            _ => panic!("Expected System command"),
        }
    }

    #[test]
    fn test_cli_parsing_all_subcommands() {
        let commands = vec![
            (vec!["sysinfo-cli", "system"], Commands::System),
            (vec!["sysinfo-cli", "cpu"], Commands::Cpu),
            (vec!["sysinfo-cli", "memory"], Commands::Memory),
            (vec!["sysinfo-cli", "disks"], Commands::Disks),
            (vec!["sysinfo-cli", "network"], Commands::Network),
            (vec!["sysinfo-cli", "components"], Commands::Components),
        ];

        for (args, expected) in commands {
            let cli = Cli::try_parse_from(args).unwrap();
            match (cli.command.unwrap(), expected) {
                (Commands::System, Commands::System) => (),
                (Commands::Cpu, Commands::Cpu) => (),
                (Commands::Memory, Commands::Memory) => (),
                (Commands::Disks, Commands::Disks) => (),
                (Commands::Network, Commands::Network) => (),
                (Commands::Components, Commands::Components) => (),
                _ => panic!("Subcommand mismatch"),
            }
        }
    }

    #[test]
    fn test_cli_parsing_processes_args() {
        let args = vec!["sysinfo-cli", "processes", "--filter", "test", "--limit", "10", "--sort", "memory"];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Commands::Processes { filter, limit, sort } = cli.command.unwrap() {
            assert_eq!(filter, Some("test".to_string()));
            assert_eq!(limit, Some(10));
            assert_eq!(sort, SortBy::Memory);
        } else {
            panic!("Expected Processes subcommand");
        }
    }

    #[test]
    fn test_format_system_info() {
        let info = SystemInfo {
            name: Some("TestOS".to_string()),
            kernel_version: Some("1.2.3".to_string()),
            os_version: Some("v1".to_string()),
            host_name: Some("test-host".to_string()),
        };
        let output = format_system_info(&info);
        assert!(output.contains("TestOS"));
        assert!(output.contains("1.2.3"));
        assert!(output.contains("v1"));
        assert!(output.contains("test-host"));
    }

    #[test]
    fn test_format_cpu_info() {
        let info = CpuInfo {
            nb_cpus: 1,
            cpus: vec![SingleCpuInfo {
                id: 0,
                usage: 50.0,
                vendor: "TestVendor".to_string(),
                brand: "TestBrand".to_string(),
            }],
            total_usage: 50.0,
        };
        let output = format_cpu_info(&info);
        assert!(output.contains("Total CPUs:"));
        assert!(output.contains("50.0%"));
        assert!(output.contains("TestVendor"));
        assert!(output.contains("TestBrand"));
    }

    #[test]
    fn test_format_memory_info() {
        let info = MemoryInfo {
            total_memory: 1024 * 1024,
            used_memory: 512 * 1024,
            total_swap: 2048 * 1024,
            used_swap: 1024 * 1024,
        };
        let output = format_memory_info(&info);
        assert!(output.contains("1.00 MiB"));
        assert!(output.contains("512.00 KiB"));
        assert!(output.contains("2.00 MiB"));
    }

    #[test]
    fn test_format_disks_info() {
        let info = vec![DiskInfo {
            name: "TestDisk".to_string(),
            kind: "SSD".to_string(),
            file_system: "ext4".to_string(),
            available_space: 100 * 1024,
            total_space: 200 * 1024,
        }];
        let output = format_disks_info(&info);
        assert!(output.contains("TestDisk"));
        assert!(output.contains("SSD"));
        assert!(output.contains("ext4"));
        assert!(output.contains("100.00 KiB"));
    }

    #[test]
    fn test_format_network_info() {
        let info = vec![NetworkInfo {
            interface: "eth0".to_string(),
            received: 1000,
            transmitted: 2000,
        }];
        let output = format_network_info(&info);
        assert!(output.contains("eth0"));
        assert!(output.contains("1000.00 B"));
        assert!(output.contains("1.95 KiB"));
    }

    #[test]
    fn test_format_components_info() {
        let info = vec![ComponentInfo {
            label: "TestTemp".to_string(),
            temperature: Some(45.5),
            max: Some(90.0),
        }];
        let output = format_components_info(&info);
        assert!(output.contains("TestTemp"));
        assert!(output.contains("45.5°C"));
        assert!(output.contains("90.0°C"));
    }

    #[test]
    fn test_format_processes_info() {
        let info = vec![ProcessInfo {
            pid: "123".to_string(),
            name: "test-proc".to_string(),
            cpu_usage: 10.0,
            memory: 1024 * 1024,
        }];
        let output = format_processes_info(&info);
        assert!(output.contains("123"));
        assert!(output.contains("test-proc"));
        assert!(output.contains("10.0"));
        assert!(output.contains("1.00 MiB"));
    }
}
