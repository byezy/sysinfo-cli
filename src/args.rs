use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "sysinfo-cli")]
#[command(about = "A useful CLI wrapper around the sysinfo crate", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output in JSON format
    #[arg(short, long, global = true)]
    pub json: bool,

    /// Refresh interval in seconds for continuous monitoring
    #[arg(short, long, global = true)]
    pub watch: Option<u64>,

    /// Save output to a file
    #[arg(short, long, global = true)]
    pub output: Option<String>,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Commands {
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
pub enum SortBy {
    Cpu,
    Memory,
    Pid,
    Name,
}

