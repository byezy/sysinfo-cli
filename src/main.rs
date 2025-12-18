mod args;
mod models;
mod collector;
mod fmt;
#[cfg(test)]
mod tests;

use clap::Parser;
use colored::*;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Write;

use crate::args::{Cli, Commands};
use crate::collector::*;
use crate::fmt::*;

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
