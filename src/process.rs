use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ProcessConsumer {
    pub name: String,
    pub short_name: String,
    pub mem: u64,
    pub count: usize,
}

pub fn get_top_consumers(limit: usize) -> Vec<ProcessConsumer> {
    let mut map: HashMap<String, (u64, usize)> = HashMap::new();

    let entries = match fs::read_dir("/proc") {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries.filter_map(Result::ok) {
        let file_name = entry.file_name();
        let pid_str = file_name.to_string_lossy();
        if !pid_str.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        let pid = &pid_str;
        let status_path = format!("/proc/{}/status", pid);
        let status_file = match File::open(&status_path) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let mut name = String::new();
        let mut rss_anon = 0u64;
        let mut vm_swap = 0u64;
        let mut vm_rss = 0u64;

        let reader = BufReader::new(status_file);
        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with("Name:") {
                name = line.split(':').nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("RssAnon:") {
                rss_anon = parse_kb_line(&line);
            } else if line.starts_with("VmSwap:") {
                vm_swap = parse_kb_line(&line);
            } else if line.starts_with("VmRSS:") {
                vm_rss = parse_kb_line(&line);
            }
        }

        if name.is_empty() {
            continue;
        }

        let mem_bytes = if rss_anon > 0 { rss_anon } else { vm_rss } + vm_swap;
        if mem_bytes < 1024 * 1024 {
            continue;
        }

        let display_name = resolve_display_name(pid, &name);

        let entry = map.entry(display_name).or_insert((0, 0));
        entry.0 += mem_bytes;
        entry.1 += 1;
    }

    let mut list: Vec<ProcessConsumer> = map
        .into_iter()
        .map(|(name, (mem, count))| {
            let short_name = if let Some((_, file)) = name.split_once(": ") {
                file.to_string()
            } else {
                name.clone()
            };
            ProcessConsumer {
                name,
                short_name,
                mem,
                count,
            }
        })
        .collect();

    list.sort_by(|a, b| b.mem.cmp(&a.mem));
    list.truncate(limit);
    list
}

fn parse_kb_line(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
        * 1024
}

fn resolve_display_name(pid: &str, name: &str) -> String {
    if !name.starts_with("python") && !matches!(name, "node" | "ruby" | "perl") {
        return name.to_string();
    }

    let cmdline_path = format!("/proc/{}/cmdline", pid);
    let mut cmdline_bytes = Vec::new();
    if let Ok(mut f) = File::open(&cmdline_path) {
        let _ = f.read_to_end(&mut cmdline_bytes);
    }

    let args: Vec<String> = cmdline_bytes
        .split(|&b| b == 0)
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| String::from_utf8_lossy(chunk).to_string())
        .collect();

    let script_arg = args.iter().skip(1).find(|arg| !arg.starts_with('-'));
    let script_arg = match script_arg {
        Some(s) => s,
        None => return name.to_string(),
    };

    let cwd = fs::read_link(format!("/proc/{}/cwd", pid)).ok();
    let full_path = if let Some(cwd) = cwd {
        if Path::new(script_arg).is_relative() {
            cwd.join(script_arg)
        } else {
            PathBuf::from(script_arg)
        }
    } else {
        PathBuf::from(script_arg)
    };

    let components: Vec<&str> = full_path
        .iter()
        .filter_map(|os| os.to_str())
        .filter(|s| !s.is_empty() && *s != "/")
        .collect();

    let folder_and_file = if components.len() >= 2 {
        format!("{}/{}", components[components.len() - 2], components[components.len() - 1])
    } else if let Some(last) = components.last() {
        last.to_string()
    } else {
        script_arg.clone()
    };

    format!("{}: {}", name, folder_and_file)
}
