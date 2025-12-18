#[cfg(test)]
mod tests {
    use crate::args::*;
    use crate::models::*;
    use crate::fmt::*;
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

