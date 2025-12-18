# sysinfo-cli

A lightweight and efficient Command Line Interface (CLI) wrapper around the [sysinfo](https://crates.io/crates/sysinfo) Rust crate. It provides a quick way to view system resources, hardware information, and running processes directly from your terminal.

## Features

- **Performance Focused**: Uses targeted refreshing to only fetch the data requested, minimizing CPU and memory overhead.
- **Memory Efficient**: Built with `default-features = false` to disable multithreading, reducing memory footprint on platforms like macOS.
- **Human Readable & Professional**: Uses `comfy-table` for beautifully formatted tables and `colored` for visual clarity.
- **JSON Support**: Global `--json` flag for machine-readable output, perfect for automation and scripting.
- **Continuous Monitoring**: Global `--watch` (or `-w`) flag to refresh data at a specified interval.
- **File Logging**: Save metrics directly to a file using the `--output` flag.

## Supported Platforms

`sysinfo-cli` supports all major operating systems. Note that in virtualized environments (Docker, WSL), some hardware-specific metrics like temperature may be unavailable.

| Platform | System Metrics | Process List | Disk/Network | UI (Colors/Tables) |
| :--- | :---: | :---: | :---: | :---: |
| **Linux** | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| **macOS** | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| **Windows** | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| **FreeBSD** | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| **Android** (Termux) | ⚠️ Partial | ⚠️ Partial | ✅ Full | ✅ Full |
| **Other Unix** (OpenBSD, etc.) | ❌ None | ❌ None | ❌ None | ✅ Full |

### Requirements
- **Rustc**: 1.88+
- **Terminal**: A terminal emulator with ANSI support (for colors and table rendering).

## Installation

### From crates.io (Recommended)
```bash
cargo install sysinfo-cli
```

### From Source
```bash
git clone https://github.com/byezy/sysinfo-cli.git
cd sysinfo-cli
cargo install --path .
```

### Manual Build
```bash
cargo build --release
# The binary will be at ./target/release/sysinfo-cli
```

## Usage

### Command Options Summary

**Global Flags:**
- `-j, --json`: Output data in JSON format.
- `-w, --watch <SECONDS>`: Refresh the display every N seconds.
- `-o, --output <FILE>`: Save the output to a specified file instead of printing to terminal.

**Subcommands:**
- `system`: Show OS name, kernel version, host name, and OS version.
- `cpu`: Show detailed per-core usage, vendor, and brand.
- `memory`: Show RAM and Swap usage.
- `disks`: List mounted disks and available space.
- `network`: Show interface statistics (received/transmitted).
- `components`: Show hardware temperatures.
- `processes`: List running processes.
    - `-f, --filter <STR>`: Filter by process name.
    - `-l, --limit <NUM>`: Limit number of results.
    - `-s, --sort <TYPE>`: Sort by `cpu`, `memory`, `pid`, or `name`.

---

## Examples

### 1. Default Summary
Running without arguments shows a high-level system overview.
```bash
sysinfo-cli
```
**Sample Output:**
```text
--- System Summary ---
System name:              "Arch Linux"
Kernel version:           "6.17.9-arch1-1"
OS version:               ""
Host name:                "thinkarch"

--- Memory Summary ---
Total memory:             62.62 GiB
Used memory:              7.10 GiB

--- CPU Summary ---
NB CPUs:                  8
Total CPU usage:          15.4%
```

### 2. System Information
```bash
sysinfo-cli system
```
**Sample Output:**
```text
System name:              "Arch Linux"
Kernel version:           "6.17.9-arch1-1"
OS version:               ""
Host name:                "thinkarch"
```

### 3. Detailed CPU Monitoring
Watch per-core usage refresh every second.
```bash
sysinfo-cli cpu --watch 1
```
**Sample Output:**
```text
=> CPUs:
Total CPUs:               8
Global usage:             12.5%
+----+---------+--------------+-------------------------------------------+
| ID | Usage % | Vendor       | Brand                                     |
+=========================================================================+
| 0  | 45.0    | GenuineIntel | Intel(R) Core(TM) i7-6820HQ CPU @ 2.70GHz |
| 1  | 2.0     | GenuineIntel | Intel(R) Core(TM) i7-6820HQ CPU @ 2.70GHz |
...
+----+---------+--------------+-------------------------------------------+
```

### 4. Memory & Swap
```bash
sysinfo-cli memory
```
**Sample Output:**
```text
Total memory:             62.62 GiB
Used memory:              7.10 GiB
Total swap:               0 B
Used swap:                0 B
```

### 5. Disk Usage
```bash
sysinfo-cli disks
```
**Sample Output:**
```text
=> Disks:
+----------------------+-----------+-------+------------+------------+
| Name                 | Kind      | FS    | Available  | Total      |
+====================================================================+
| /dev/nvme0n1p3       | SSD       | btrfs | 150.30 GiB | 450.00 GiB |
| /dev/sda1            | HDD       | ext4  | 2.10 TiB   | 4.00 TiB   |
+----------------------+-----------+-------+------------+------------+
```

### 6. Network Statistics
```bash
sysinfo-cli network
```
**Sample Output:**
```text
=> Networks:
+-----------+------------+-------------+
| Interface | Received   | Transmitted |
+======================================+
| eth0      | 14.20 MiB  | 1.50 MiB    |
| lo        | 1.20 KiB   | 1.20 KiB    |
| wlan0     | 450.30 MiB | 45.10 MiB   |
+-----------+------------+-------------+
```

### 7. Hardware Components (Temperature)
```bash
sysinfo-cli components
```
**Sample Output:**
```text
=> Components:
+-----------------------+---------+---------+
| Label                 | Temp    | Max     |
+===========================================+
| Core 0                | 45.0°C  | 90.0°C  |
| Core 1                | 46.0°C  | 90.0°C  |
| Composite             | 38.0°C  | 70.0°C  |
+-----------------------+---------+---------+
```

### 8. Process Management
Find the top 5 memory-hungry processes.
```bash
sysinfo-cli processes --sort memory --limit 5
```
**Sample Output:**
```text
=> Processes:
+--------+-----------------+-------+------------+
| PID    | Name            | CPU % | Memory     |
+===============================================+
| 1426   | firefox         |  5.2  | 727.64 MiB |
| 186924 | vivaldi-bin     |  1.0  | 512.20 MiB |
| 1245   | Xwayland        |  0.5  | 236.11 MiB |
| 2145   | rust-analyzer   |  0.2  | 180.45 MiB |
| 3210   | gnome-shell     |  2.1  | 150.30 MiB |
+--------+-----------------+-------+------------+
```

### 9. JSON Export & File Logging
Save network statistics to a JSON file for processing.
```bash
sysinfo-cli --json --output net_stats.json network
```
**Sample Output (`net_stats.json`):**
```json
[
  {
    "interface": "wlan0",
    "received": 14567890,
    "transmitted": 8901234
  },
  {
    "interface": "lo",
    "received": 1024,
    "transmitted": 1024
  }
]
```

## License

This project is licensed under the MIT License.
