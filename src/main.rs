use clap::Parser;
use colored::*;
use std::{thread, time::Duration};
use sysinfo::System;

#[derive(Parser)]
#[command(name = "memory-monitor")]
#[command(about = "A system monitor showing memory usage and processes")]
struct Args {
    /// Update interval in seconds
    #[arg(short, long, default_value_t = 1)]
    interval: u64,

    /// Number of updates before exiting (0 for infinite)
    #[arg(short, long, default_value_t = 0)]
    count: u32,

    /// Number of processes to show
    #[arg(short, long, default_value_t = 10)]
    top: usize,

    /// Sort by memory usage instead of CPU
    #[arg(short, long)]
    memory_sort: bool,
}

struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory: u64,
    memory_percent: f32,
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes = bytes as f64;
    if bytes >= GB {
        format!("{:.2} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes / KB)
    } else {
        format!("{} B", bytes)
    }
}

fn get_color_for_percentage(percent: f64) -> Color {
    match percent as u64 {
        0..=60 => Color::Green,
        61..=85 => Color::Yellow,
        _ => Color::Red,
    }
}

fn print_table(processes: &[ProcessInfo]) {
    // Define column widths
    let pid_width = 8;
    let name_width = 20;
    let cpu_width = 10;
    let mem_width = 12;
    let mem_percent_width = 10;

    // Print header
    println!(
        "{:<pid_width$} {:<name_width$} {:>cpu_width$} {:>mem_width$} {:>mem_percent_width$}",
        "PID",
        "NAME",
        "CPU %",
        "MEMORY",
        "MEM %",
        pid_width = pid_width,
        name_width = name_width,
        cpu_width = cpu_width,
        mem_width = mem_width,
        mem_percent_width = mem_percent_width
    );

    // Print separator
    println!(
        "{}",
        "─".repeat(pid_width + name_width + cpu_width + mem_width + mem_percent_width + 4)
    );

    // Print processes
    for proc in processes {
        let name = if proc.name.len() > name_width {
            format!("{}...", &proc.name[..name_width - 3])
        } else {
            proc.name.clone()
        };

        println!(
            "{:<pid_width$} {:<name_width$} {:>cpu_width$.1} {:>mem_width$} {:>mem_percent_width$.1}%",
            proc.pid,
            name,
            proc.cpu_usage,
            format_bytes(proc.memory),
            proc.memory_percent,
            pid_width=pid_width,
            name_width=name_width,
            cpu_width=cpu_width,
            mem_width=mem_width,
            mem_percent_width=mem_percent_width
        );
    }
}

fn main() {
    let args = Args::parse();
    let mut sys = System::new_all();
    let mut updates = 0;

    loop {
        // Refresh all system information
        sys.refresh_all();

        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let mem_percent = (used_mem as f64 / total_mem as f64 * 100.0) as u64;

        // Clear screen and move cursor to top
        print!("\x1B[2J\x1B[1;1H");

        // Display system memory information
        println!("{}", "System Memory".green().bold());
        println!("{}", "─".repeat(40));
        println!("{}: {}", "Total".blue(), format_bytes(total_mem).white());
        println!("{}: {}", "Used".blue(), format_bytes(used_mem).white());
        println!(
            "{}: {}%",
            "Percentage".blue(),
            mem_percent
                .to_string()
                .color(get_color_for_percentage(mem_percent as f64))
        );
        println!();

        // Process list
        println!("{}", "Top Processes".green().bold());
        println!("{}", "─".repeat(40));

        let mut processes: Vec<ProcessInfo> = sys
            .processes()
            .values()
            .map(|proc| ProcessInfo {
                pid: proc.pid().as_u32(),
                name: proc.name().to_str().unwrap().to_string(),
                cpu_usage: proc.cpu_usage(),
                memory: proc.memory(),
                memory_percent: (proc.memory() as f64 / total_mem as f64 * 100.0) as f32,
            })
            .collect();

        // Sort processes
        if args.memory_sort {
            processes.sort_by(|a, b| b.memory_percent.partial_cmp(&a.memory_percent).unwrap());
        } else {
            processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
        }

        // Take only the top N processes
        processes.truncate(args.top);

        // Display process table
        print_table(&processes);

        updates += 1;
        if args.count > 0 && updates >= args.count {
            break;
        }

        thread::sleep(Duration::from_secs(args.interval));
    }
}
