use colored::*;
use comfy_table::Table;
use crate::models::*;

pub fn format_system_info(info: &SystemInfo) -> String {
    let mut s = String::new();
    s.push_str(&format!("{:<25} {:?}\n", "System name:".yellow(), info.name.as_deref().unwrap_or_default()));
    s.push_str(&format!("{:<25} {:?}\n", "Kernel version:".yellow(), info.kernel_version.as_deref().unwrap_or_default()));
    s.push_str(&format!("{:<25} {:?}\n", "OS version:".yellow(), info.os_version.as_deref().unwrap_or_default()));
    s.push_str(&format!("{:<25} {:?}\n", "Host name:".yellow(), info.host_name.as_deref().unwrap_or_default()));
    s
}

pub fn format_cpu_info(info: &CpuInfo) -> String {
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

pub fn format_memory_info(info: &MemoryInfo) -> String {
    let mut s = String::new();
    s.push_str(&format!("{:<25} {}\n", "Total memory:".yellow(), format_bytes(info.total_memory)));
    s.push_str(&format!("{:<25} {}\n", "Used memory:".yellow(), format_bytes(info.used_memory)));
    s.push_str(&format!("{:<25} {}\n", "Total swap:".yellow(), format_bytes(info.total_swap)));
    s.push_str(&format!("{:<25} {}\n", "Used swap:".yellow(), format_bytes(info.used_swap)));
    s
}

pub fn format_disks_info(info: &[DiskInfo]) -> String {
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

pub fn format_network_info(info: &[NetworkInfo]) -> String {
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

pub fn format_components_info(info: &[ComponentInfo]) -> String {
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

pub fn format_processes_info(info: &[ProcessInfo]) -> String {
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

pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let i = (bytes as f64).log(1024.0).floor() as usize;
    let i = i.min(units.len() - 1);
    let value = bytes as f64 / 1024.0f64.powi(i as i32);
    format!("{:.2} {}", value, units[i])
}

