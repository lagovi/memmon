use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone, Default)]
pub struct SystemMemory {
    pub total_ram: u64,
    pub avail_ram: u64,
    pub total_cache: u64,
    pub ssd_total: u64,
    pub ssd_used: u64,
    pub total_unified: u64,
    pub total_avail: u64,
    pub free_real: u64,
}

pub fn get_system_memory() -> SystemMemory {
    let mut total_ram = 1u64;
    let mut free_ram = 0u64;
    let mut avail_ram = 0u64;
    let mut buffers = 0u64;
    let mut cached = 0u64;
    let mut slab = 0u64;

    if let Ok(file) = File::open("/proc/meminfo") {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            let mut parts = line.split(':');
            if let (Some(key), Some(val_part)) = (parts.next(), parts.next()) {
                let key = key.trim();
                let bytes = val_part
                    .trim()
                    .split_whitespace()
                    .next()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0)
                    * 1024;

                match key {
                    "MemTotal" => total_ram = bytes,
                    "MemFree" => free_ram = bytes,
                    "MemAvailable" => avail_ram = bytes,
                    "Buffers" => buffers = bytes,
                    "Cached" => cached = bytes,
                    "Slab" => slab = bytes,
                    _ => {}
                }
            }
        }
    }

    let mut ssd_total = 0u64;
    let mut ssd_used = 0u64;

    if let Ok(file) = File::open("/proc/swaps") {
        let reader = BufReader::new(file);
        for line in reader.lines().skip(1).map_while(Result::ok) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 && !parts[0].contains("zram") {
                let size = parts[2].parse::<u64>().unwrap_or(0) * 1024;
                let used = parts[3].parse::<u64>().unwrap_or(0) * 1024;
                ssd_total += size;
                ssd_used += used;
            }
        }
    }

    let total_cache = buffers + cached + slab;
    let ssd_free = ssd_total.saturating_sub(ssd_used);
    let total_unified = total_ram + ssd_total;
    let total_avail = avail_ram + ssd_free;
    let free_real = free_ram + ssd_free;

    SystemMemory {
        total_ram,
        avail_ram,
        total_cache,
        ssd_total,
        ssd_used,
        total_unified,
        total_avail,
        free_real,
    }
}
