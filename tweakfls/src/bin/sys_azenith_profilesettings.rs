use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::path::Path;
use std::collections::HashSet;

fn getprop(prop_name: &str) -> String {
    if let Ok(output) = Command::new("getprop").arg(prop_name).output() {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    String::new()
}

fn setprop(prop_name: &str, value: &str) {
    let _ = Command::new("setprop").arg(prop_name).arg(value).output();
}

fn az_log(message: &str) {
    if getprop("persist.sys.azenith.debugmode") == "true" {
        let _ = Command::new("sys.azenith-service")
            .arg("--verboselog")
            .arg("AZLog")
            .arg("0")
            .arg(message)
            .output();
    }
}

fn dlog(message: &str) {
    let _ = Command::new("sys.azenith-service")
        .arg("--log")
        .arg("AZenith")
        .arg("1")
        .arg(message)
        .output();
}

fn zeshia(value: &str, path: &str, lock: bool) {
    let p = Path::new(path);
    let pathname = p
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    if !p.exists() {
        az_log(&format!("File /{} not found, skipping...", pathname));
        return;
    }

    if let Ok(mut perms) = fs::metadata(p).map(|m| m.permissions()) {
        perms.set_mode(0o644);
        let _ = fs::set_permissions(p, perms);
    }

    if let Ok(mut file) = OpenOptions::new().write(true).truncate(true).open(p) {
        if file.write_all(value.as_bytes()).is_ok() {
            az_log(&format!("Set /{} to {}", pathname, value));
        } else {
            az_log(&format!("Cannot write to /{} (permission denied)", pathname));
        }
    }

    if lock {
        if let Ok(mut perms) = fs::metadata(p).map(|m| m.permissions()) {
            perms.set_mode(0o444);
            let _ = fs::set_permissions(p, perms);
        }
    }
}

fn applyppmnfreqsets(val: &str, path: &str) -> bool {
    let p = Path::new(path);
    if !p.is_file() {
        return false;
    }
    if let Ok(mut perms) = fs::metadata(p).map(|m| m.permissions()) {
        perms.set_mode(0o644);
        let _ = fs::set_permissions(p, perms);
    }
    let _ = fs::write(p, val);
    if let Ok(mut perms) = fs::metadata(p).map(|m| m.permissions()) {
        perms.set_mode(0o444);
        let _ = fs::set_permissions(p, perms);
    }
    true
}

fn get_freqs(path: &str) -> Vec<i64> {
    if let Ok(content) = fs::read_to_string(path) {
        let mut freqs: Vec<i64> = content
            .split_whitespace()
            .filter_map(|s| s.parse::<i64>().ok())
            .collect();
        freqs.sort_unstable();
        return freqs;
    }
    Vec::new()
}

fn which_maxfreq(path: &str) -> String {
    let freqs = get_freqs(path);
    if let Some(&max) = freqs.last() {
        return max.to_string();
    }
    String::new()
}

fn which_minfreq(path: &str) -> String {
    let freqs = get_freqs(path);
    if let Some(&min) = freqs.first() {
        return min.to_string();
    }
    String::new()
}

fn which_midfreq(path: &str) -> String {
    let freqs = get_freqs(path);
    let len = freqs.len();
    if len > 0 {
        let mid = len / 2;
        return freqs[mid].to_string();
    }
    String::new()
}

fn setfreqs(file: &str, target: &str) -> String {
    if Path::new(file).exists() {
        if let Ok(target_val) = target.parse::<i64>() {
            let freqs = get_freqs(file);
            let mut best = target_val;
            let mut min_diff = i64::MAX;
            for &freq in &freqs {
                let diff = (target_val - freq).abs();
                if diff < min_diff {
                    min_diff = diff;
                    best = freq;
                }
            }
            return best.to_string();
        }
    }
    target.to_string()
}

fn devfreq_max_perf(path: &str) {
    let avail = format!("{}/available_frequencies", path);
    if !Path::new(&avail).exists() { return; }
    let max_freq = which_maxfreq(&avail);
    zeshia(&max_freq, &format!("{}/max_freq", path), true);
    zeshia(&max_freq, &format!("{}/min_freq", path), true);
}

fn devfreq_mid_perf(path: &str) {
    let avail = format!("{}/available_frequencies", path);
    if !Path::new(&avail).exists() { return; }
    let max_freq = which_maxfreq(&avail);
    let mid_freq = which_midfreq(&avail);
    zeshia(&max_freq, &format!("{}/max_freq", path), true);
    zeshia(&mid_freq, &format!("{}/min_freq", path), true);
}

fn devfreq_unlock(path: &str) {
    let avail = format!("{}/available_frequencies", path);
    if !Path::new(&avail).exists() { return; }
    let max_freq = which_maxfreq(&avail);
    let min_freq = which_minfreq(&avail);
    zeshia(&max_freq, &format!("{}/max_freq", path), false);
    zeshia(&min_freq, &format!("{}/min_freq", path), false);
}

fn devfreq_min_perf(path: &str) {
    let avail = format!("{}/available_frequencies", path);
    if !Path::new(&avail).exists() { return; }
    let freq = which_minfreq(&avail);
    zeshia(&freq, &format!("{}/min_freq", path), true);
    zeshia(&freq, &format!("{}/max_freq", path), true);
}

fn qcom_cpudcvs_max_perf(path: &str) {
    let avail = format!("{}/available_frequencies", path);
    if !Path::new(&avail).exists() { return; }
    let freq = which_maxfreq(&avail);
    zeshia(&freq, &format!("{}/hw_max_freq", path), true);
    zeshia(&freq, &format!("{}/hw_min_freq", path), true);
}

fn qcom_cpudcvs_mid_perf(path: &str) {
    let avail = format!("{}/available_frequencies", path);
    if !Path::new(&avail).exists() { return; }
    let max_freq = which_maxfreq(&avail);
    let mid_freq = which_midfreq(&avail);
    zeshia(&max_freq, &format!("{}/hw_max_freq", path), true);
    zeshia(&mid_freq, &format!("{}/hw_min_freq", path), true);
}

fn qcom_cpudcvs_unlock(path: &str) {
    let avail = format!("{}/available_frequencies", path);
    if !Path::new(&avail).exists() { return; }
    let max_freq = which_maxfreq(&avail);
    let min_freq = which_minfreq(&avail);
    zeshia(&max_freq, &format!("{}/hw_max_freq", path), false);
    zeshia(&min_freq, &format!("{}/hw_min_freq", path), false);
}

fn qcom_cpudcvs_min_perf(path: &str) {
    let avail = format!("{}/available_frequencies", path);
    if !Path::new(&avail).exists() { return; }
    let freq = which_minfreq(&avail);
    zeshia(&freq, &format!("{}/hw_min_freq", path), true);
    zeshia(&freq, &format!("{}/hw_max_freq", path), true);
}

fn setgov(gov: &str) {
    if let Ok(entries) = glob::glob("/sys/devices/system/cpu/cpu*/cpufreq/scaling_governor") {
        for entry in entries.flatten() {
            let path_str = entry.to_string_lossy();
            if let Ok(mut perms) = fs::metadata(&*path_str).map(|m| m.permissions()) {
                perms.set_mode(0o644);
                let _ = fs::set_permissions(&*path_str, perms);
            }
            if let Ok(mut file) = OpenOptions::new().write(true).truncate(true).open(&*path_str) {
                let _ = file.write_all(gov.as_bytes());
            }
            if let Ok(mut perms) = fs::metadata(&*path_str).map(|m| m.permissions()) {
                perms.set_mode(0o444);
                let _ = fs::set_permissions(&*path_str, perms);
            }
        }
    }

    if let Ok(entries) = glob::glob("/sys/devices/system/cpu/cpufreq/policy*/scaling_governor") {
        for entry in entries.flatten() {
            let path_str = entry.to_string_lossy();
            if let Ok(mut perms) = fs::metadata(&*path_str).map(|m| m.permissions()) {
                perms.set_mode(0o444);
                let _ = fs::set_permissions(&*path_str, perms);
            }
        }
    }
}

fn sets_io(scheduler: &str) {
    let blocks = ["sda", "sdb", "sdc", "mmcblk0", "mmcblk1"];
    for block in blocks.iter() {
        let path_str = format!("/sys/block/{}/queue/scheduler", block);
        let p = Path::new(&path_str);
        if p.exists() {
            if let Ok(mut perms) = fs::metadata(p).map(|m| m.permissions()) {
                perms.set_mode(0o644);
                let _ = fs::set_permissions(p, perms);
            }
            if let Ok(mut file) = OpenOptions::new().write(true).truncate(true).open(p) {
                let _ = file.write_all(scheduler.as_bytes());
            }
            if let Ok(mut perms) = fs::metadata(p).map(|m| m.permissions()) {
                perms.set_mode(0o444);
                let _ = fs::set_permissions(p, perms);
            }
        }
    }
}

fn sets_gpu_mali(gov: &str) {
    if let Ok(mali_entries) = glob::glob("/sys/devices/platform/soc/*.mali") {
        for mali_entry in mali_entries.flatten() {
            let mali_path = mali_entry.to_string_lossy();
            let pattern = format!("{}/devfreq/*.mali/governor", mali_path);
            if let Ok(gov_entries) = glob::glob(&pattern) {
                for gov_entry in gov_entries.flatten() {
                    let gov_path = gov_entry.to_string_lossy();
                    if let Ok(mut perms) = fs::metadata(&*gov_path).map(|m| m.permissions()) {
                        perms.set_mode(0o644);
                        let _ = fs::set_permissions(&*gov_path, perms);
                    }
                    if let Ok(mut file) = OpenOptions::new().write(true).truncate(true).open(&*gov_path) {
                        let _ = file.write_all(gov.as_bytes());
                    }
                    if let Ok(mut perms) = fs::metadata(&*gov_path).map(|m| m.permissions()) {
                        perms.set_mode(0o444);
                        let _ = fs::set_permissions(&*gov_path, perms);
                    }
                }
            }
        }
    }
}

fn get_freq_limiter() -> i64 {
    let raw = getprop("persist.sys.azenithconf.freqoffset");
    if raw.is_empty() { return 100; }
    let raw = raw.replace("Disabled", "100").replace("%", "");
    raw.parse::<i64>().unwrap_or(100)
}

fn setfreqppm() {
    if Path::new("/proc/ppm").exists() {
        let limiter = get_freq_limiter();
        let curprofile = fs::read_to_string("/data/adb/.config/AZenith/API/current_profile")
            .unwrap_or_default().trim().to_string();

        let mut cluster = 0;
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/cpufreq/policy*") {
            for entry in entries.flatten() {
                let path_str = entry.to_string_lossy();
                let cpu_maxfreq_str = fs::read_to_string(format!("{}/cpuinfo_max_freq", path_str)).unwrap_or_default().trim().to_string();
                let cpu_minfreq_str = fs::read_to_string(format!("{}/cpuinfo_min_freq", path_str)).unwrap_or_default().trim().to_string();

                let cpu_maxfreq = cpu_maxfreq_str.parse::<i64>().unwrap_or(0);
                let cpu_minfreq = cpu_minfreq_str.parse::<i64>().unwrap_or(0);

                let new_max_target = cpu_maxfreq * limiter / 100;
                let new_maxfreq = setfreqs(&format!("{}/scaling_available_frequencies", path_str), &new_max_target.to_string());

                let policy_name = entry.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();

                if curprofile == "3" {
                    let target_min_target = cpu_maxfreq * 40 / 100;
                    let new_minfreq = setfreqs(&format!("{}/scaling_available_frequencies", path_str), &target_min_target.to_string());
                    zeshia(&format!("{} {}", cluster, new_maxfreq), "/proc/ppm/policy/hard_userlimit_max_cpu_freq", true);
                    zeshia(&format!("{} {}", cluster, new_minfreq), "/proc/ppm/policy/hard_userlimit_min_cpu_freq", true);
                    dlog(&format!("Set {} maxfreq={} minfreq={}", policy_name, new_maxfreq, new_minfreq));
                } else {
                    zeshia(&format!("{} {}", cluster, new_maxfreq), "/proc/ppm/policy/hard_userlimit_max_cpu_freq", true);
                    zeshia(&format!("{} {}", cluster, cpu_minfreq), "/proc/ppm/policy/hard_userlimit_min_cpu_freq", true);
                    dlog(&format!("Set {} maxfreq={} minfreq={}", policy_name, new_maxfreq, cpu_minfreq));
                }
                cluster += 1;
            }
        }
    }
}

fn clear_background_apps() {
    let out = Command::new("dumpsys").args(&["window", "displays"]).output();
    if let Ok(out) = out {
        let s = String::from_utf8_lossy(&out.stdout);
        let mut visible_pkgs = HashSet::new();
        let mut invisible_pkgs = HashSet::new();

        for line in s.lines() {
            if line.contains("Task{") {
                if let Some(pkg) = extract_pkg(line) {
                    if line.contains("visible=true") {
                        visible_pkgs.insert(pkg.clone());
                    } else if line.contains("visible=false") {
                        invisible_pkgs.insert(pkg.clone());
                    }
                }
            }
        }

        for pkg in invisible_pkgs {
            if !visible_pkgs.contains(&pkg) {
                if !pkg.contains("com.android.systemui") && !pkg.contains("com.android.settings") && !pkg.contains("android") && !pkg.contains("system") {
                    let _ = Command::new("am").args(&["force-stop", &pkg]).output();
                    az_log(&format!("Stopped app: {}", pkg));
                }
            }
        }
        dlog("Cleared background apps");
    }
}

fn extract_pkg(line: &str) -> Option<String> {
    if let Some(pos) = line.find("A=") {
        let rest = &line[pos+2..];
        if let Some(colon) = rest.find(':') {
            let after_colon = &rest[colon+1..];
            let end = after_colon.find(|c: char| c.is_whitespace() || c == ' ' || c == '}').unwrap_or(after_colon.len());
            return Some(after_colon[..end].to_string());
        }
    }
    None
}

fn setfreq() {
    let limiter = get_freq_limiter();
    let curprofile = fs::read_to_string("/data/adb/.config/AZenith/API/current_profile")
        .unwrap_or_default().trim().to_string();

    if let Ok(entries) = glob::glob("/sys/devices/system/cpu/*/cpufreq") {
        for entry in entries.flatten() {
            let path_str = entry.to_string_lossy();
            let cpu_maxfreq_str = fs::read_to_string(format!("{}/cpuinfo_max_freq", path_str)).unwrap_or_default().trim().to_string();
            let cpu_minfreq_str = fs::read_to_string(format!("{}/cpuinfo_min_freq", path_str)).unwrap_or_default().trim().to_string();

            let cpu_maxfreq = cpu_maxfreq_str.parse::<i64>().unwrap_or(0);
            let cpu_minfreq = cpu_minfreq_str.parse::<i64>().unwrap_or(0);

            let new_max_target = cpu_maxfreq * limiter / 100;
            let new_maxfreq = setfreqs(&format!("{}/scaling_available_frequencies", path_str), &new_max_target.to_string());

            let policy_name = entry.parent().and_then(|p| p.file_name()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();

            if curprofile == "3" {
                let target_min_target = cpu_maxfreq * 40 / 100;
                let new_minfreq = setfreqs(&format!("{}/scaling_available_frequencies", path_str), &target_min_target.to_string());
                zeshia(&new_maxfreq, &format!("{}/scaling_max_freq", path_str), true);
                zeshia(&new_minfreq, &format!("{}/scaling_min_freq", path_str), true);
                dlog(&format!("Set {} maxfreq={} minfreq={}", policy_name, new_maxfreq, new_minfreq));
            } else {
                zeshia(&new_maxfreq, &format!("{}/scaling_max_freq", path_str), true);
                zeshia(&cpu_minfreq.to_string(), &format!("{}/scaling_min_freq", path_str), true);
                dlog(&format!("Set {} maxfreq={} minfreq={}", policy_name, new_maxfreq, cpu_minfreq));
            }
        }
    }

    if let Ok(entries) = glob::glob("/sys/devices/system/cpu/cpufreq/policy*/scaling_*_freq") {
        for entry in entries.flatten() {
            let path_str = entry.to_string_lossy();
            if let Ok(mut perms) = fs::metadata(&*path_str).map(|m| m.permissions()) {
                perms.set_mode(0o444);
                let _ = fs::set_permissions(&*path_str, perms);
            }
        }
    }
}

fn setgamefreqppm() {
    if Path::new("/proc/ppm").exists() {
        let litemode = getprop("persist.sys.azenithconf.litemode");
        let litemode_val = litemode.parse::<i64>().unwrap_or(0);

        let mut cluster = -1;
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/cpufreq/policy*") {
            for entry in entries.flatten() {
                cluster += 1;
                let path_str = entry.to_string_lossy();
                let cpu_maxfreq_str = fs::read_to_string(format!("{}/cpuinfo_max_freq", path_str)).unwrap_or_default().trim().to_string();
                let cpu_maxfreq = cpu_maxfreq_str.parse::<i64>().unwrap_or(0);

                let new_midtarget = cpu_maxfreq * 100 / 100;
                let new_midfreq = setfreqs(&format!("{}/scaling_available_frequencies", path_str), &new_midtarget.to_string());

                let policy_name = entry.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();

                if litemode_val == 1 {
                    let cpu_minfreq_str = fs::read_to_string(format!("{}/cpuinfo_min_freq", path_str)).unwrap_or_default().trim().to_string();
                    zeshia(&format!("{} {}", cluster, new_midfreq), "/proc/ppm/policy/hard_userlimit_max_cpu_freq", true);
                    zeshia(&format!("{} {}", cluster, cpu_minfreq_str), "/proc/ppm/policy/hard_userlimit_min_cpu_freq", true);
                    dlog(&format!("Set {} maxfreq={} minfreq={}", policy_name, new_midfreq, cpu_minfreq_str));
                } else {
                    zeshia(&format!("{} {}", cluster, cpu_maxfreq), "/proc/ppm/policy/hard_userlimit_max_cpu_freq", true);
                    zeshia(&format!("{} {}", cluster, new_midfreq), "/proc/ppm/policy/hard_userlimit_min_cpu_freq", true);
                    dlog(&format!("Set {} maxfreq={} minfreq={}", policy_name, cpu_maxfreq, new_midfreq));
                }
            }
        }
    }
}

fn setgamefreq() {
    let litemode = getprop("persist.sys.azenithconf.litemode");
    let litemode_val = litemode.parse::<i64>().unwrap_or(0);

    if let Ok(entries) = glob::glob("/sys/devices/system/cpu/*/cpufreq") {
        for entry in entries.flatten() {
            let path_str = entry.to_string_lossy();
            let cpu_maxfreq_str = fs::read_to_string(format!("{}/cpuinfo_max_freq", path_str)).unwrap_or_default().trim().to_string();
            let cpu_maxfreq = cpu_maxfreq_str.parse::<i64>().unwrap_or(0);

            let new_midtarget = cpu_maxfreq * 100 / 100;
            let new_midfreq = setfreqs(&format!("{}/scaling_available_frequencies", path_str), &new_midtarget.to_string());

            let policy_name = entry.parent().and_then(|p| p.file_name()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();

            if litemode_val == 1 {
                let cpu_minfreq_str = fs::read_to_string(format!("{}/cpuinfo_min_freq", path_str)).unwrap_or_default().trim().to_string();
                zeshia(&new_midfreq, &format!("{}/scaling_max_freq", path_str), true);
                zeshia(&cpu_minfreq_str, &format!("{}/scaling_min_freq", path_str), true);
                dlog(&format!("Set {} maxfreq={} minfreq={}", policy_name, new_midfreq, cpu_minfreq_str));
            } else {
                zeshia(&cpu_maxfreq_str, &format!("{}/scaling_max_freq", path_str), true);
                zeshia(&new_midfreq, &format!("{}/scaling_min_freq", path_str), true);
                dlog(&format!("Set {} maxfreq={} minfreq={}", policy_name, cpu_maxfreq, new_midfreq));
            }
        }
    }

    if let Ok(entries) = glob::glob("/sys/devices/system/cpu/cpufreq/policy*/scaling_*_freq") {
        for entry in entries.flatten() {
            let path_str = entry.to_string_lossy();
            if let Ok(mut perms) = fs::metadata(&*path_str).map(|m| m.permissions()) {
                perms.set_mode(0o444);
                let _ = fs::set_permissions(&*path_str, perms);
            }
        }
    }
}

fn dsetfreqppm() {
    if Path::new("/proc/ppm").exists() {
        let limiter = get_freq_limiter();
        let curprofile = fs::read_to_string("/data/adb/.config/AZenith/API/current_profile")
            .unwrap_or_default().trim().to_string();

        let mut cluster = 0;
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/cpufreq/policy*") {
            for entry in entries.flatten() {
                let path_str = entry.to_string_lossy();
                let cpu_maxfreq_str = fs::read_to_string(format!("{}/cpuinfo_max_freq", path_str)).unwrap_or_default().trim().to_string();
                let cpu_minfreq_str = fs::read_to_string(format!("{}/cpuinfo_min_freq", path_str)).unwrap_or_default().trim().to_string();

                let cpu_maxfreq = cpu_maxfreq_str.parse::<i64>().unwrap_or(0);

                let new_max_target = cpu_maxfreq * limiter / 100;
                let new_maxfreq = setfreqs(&format!("{}/scaling_available_frequencies", path_str), &new_max_target.to_string());

                if curprofile == "3" {
                    let target_min_target = cpu_maxfreq * 40 / 100;
                    let new_minfreq = setfreqs(&format!("{}/scaling_available_frequencies", path_str), &target_min_target.to_string());
                    applyppmnfreqsets(&format!("{} {}", cluster, new_maxfreq), "/proc/ppm/policy/hard_userlimit_max_cpu_freq");
                    applyppmnfreqsets(&format!("{} {}", cluster, new_minfreq), "/proc/ppm/policy/hard_userlimit_min_cpu_freq");
                } else {
                    applyppmnfreqsets(&format!("{} {}", cluster, new_maxfreq), "/proc/ppm/policy/hard_userlimit_max_cpu_freq");
                    applyppmnfreqsets(&format!("{} {}", cluster, cpu_minfreq_str), "/proc/ppm/policy/hard_userlimit_min_cpu_freq");
                }
                cluster += 1;
            }
        }
    }
}

fn dsetfreq() {
    let limiter = get_freq_limiter();
    let curprofile = fs::read_to_string("/data/adb/.config/AZenith/API/current_profile")
        .unwrap_or_default().trim().to_string();

    if let Ok(entries) = glob::glob("/sys/devices/system/cpu/*/cpufreq") {
        for entry in entries.flatten() {
            let path_str = entry.to_string_lossy();
            let cpu_maxfreq_str = fs::read_to_string(format!("{}/cpuinfo_max_freq", path_str)).unwrap_or_default().trim().to_string();
            let cpu_minfreq_str = fs::read_to_string(format!("{}/cpuinfo_min_freq", path_str)).unwrap_or_default().trim().to_string();

            let cpu_maxfreq = cpu_maxfreq_str.parse::<i64>().unwrap_or(0);

            let new_max_target = cpu_maxfreq * limiter / 100;
            let new_maxfreq = setfreqs(&format!("{}/scaling_available_frequencies", path_str), &new_max_target.to_string());

            if curprofile == "3" {
                let target_min_target = cpu_maxfreq * 40 / 100;
                let new_minfreq = setfreqs(&format!("{}/scaling_available_frequencies", path_str), &target_min_target.to_string());
                applyppmnfreqsets(&new_maxfreq, &format!("{}/scaling_max_freq", path_str));
                applyppmnfreqsets(&new_minfreq, &format!("{}/scaling_min_freq", path_str));
            } else {
                applyppmnfreqsets(&new_maxfreq, &format!("{}/scaling_max_freq", path_str));
                applyppmnfreqsets(&cpu_minfreq_str, &format!("{}/scaling_min_freq", path_str));
            }
        }
    }

    if let Ok(entries) = glob::glob("/sys/devices/system/cpu/cpufreq/policy*/scaling_*_freq") {
        for entry in entries.flatten() {
            let path_str = entry.to_string_lossy();
            if let Ok(mut perms) = fs::metadata(&*path_str).map(|m| m.permissions()) {
                perms.set_mode(0o444);
                let _ = fs::set_permissions(&*path_str, perms);
            }
        }
    }
}

fn dsetgamefreqppm() {
    if Path::new("/proc/ppm").exists() {
        let litemode = getprop("persist.sys.azenithconf.litemode");
        let litemode_val = litemode.parse::<i64>().unwrap_or(0);

        let mut cluster = -1;
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/cpufreq/policy*") {
            for entry in entries.flatten() {
                cluster += 1;
                let path_str = entry.to_string_lossy();
                let cpu_maxfreq_str = fs::read_to_string(format!("{}/cpuinfo_max_freq", path_str)).unwrap_or_default().trim().to_string();
                let cpu_maxfreq = cpu_maxfreq_str.parse::<i64>().unwrap_or(0);

                let new_midtarget = cpu_maxfreq * 100 / 100;
                let new_midfreq = setfreqs(&format!("{}/scaling_available_frequencies", path_str), &new_midtarget.to_string());

                if litemode_val == 1 {
                    let cpu_minfreq_str = fs::read_to_string(format!("{}/cpuinfo_min_freq", path_str)).unwrap_or_default().trim().to_string();
                    applyppmnfreqsets(&format!("{} {}", cluster, new_midfreq), "/proc/ppm/policy/hard_userlimit_max_cpu_freq");
                    applyppmnfreqsets(&format!("{} {}", cluster, cpu_minfreq_str), "/proc/ppm/policy/hard_userlimit_min_cpu_freq");
                } else {
                    applyppmnfreqsets(&format!("{} {}", cluster, cpu_maxfreq), "/proc/ppm/policy/hard_userlimit_max_cpu_freq");
                    applyppmnfreqsets(&format!("{} {}", cluster, new_midfreq), "/proc/ppm/policy/hard_userlimit_min_cpu_freq");
                }
            }
        }
    }
}

fn dsetgamefreq() {
    let litemode = getprop("persist.sys.azenithconf.litemode");
    let litemode_val = litemode.parse::<i64>().unwrap_or(0);

    if let Ok(entries) = glob::glob("/sys/devices/system/cpu/*/cpufreq") {
        for entry in entries.flatten() {
            let path_str = entry.to_string_lossy();
            let cpu_maxfreq_str = fs::read_to_string(format!("{}/cpuinfo_max_freq", path_str)).unwrap_or_default().trim().to_string();
            let cpu_maxfreq = cpu_maxfreq_str.parse::<i64>().unwrap_or(0);

            let new_midtarget = cpu_maxfreq * 100 / 100;
            let new_midfreq = setfreqs(&format!("{}/scaling_available_frequencies", path_str), &new_midtarget.to_string());

            if litemode_val == 1 {
                let cpu_minfreq_str = fs::read_to_string(format!("{}/cpuinfo_min_freq", path_str)).unwrap_or_default().trim().to_string();
                applyppmnfreqsets(&new_midfreq, &format!("{}/scaling_max_freq", path_str));
                applyppmnfreqsets(&cpu_minfreq_str, &format!("{}/scaling_min_freq", path_str));
            } else {
                applyppmnfreqsets(&cpu_maxfreq_str, &format!("{}/scaling_max_freq", path_str));
                applyppmnfreqsets(&new_midfreq, &format!("{}/scaling_min_freq", path_str));
            }
        }
    }

    if let Ok(entries) = glob::glob("/sys/devices/system/cpu/cpufreq/policy*/scaling_*_freq") {
        for entry in entries.flatten() {
            let path_str = entry.to_string_lossy();
            if let Ok(mut perms) = fs::metadata(&*path_str).map(|m| m.permissions()) {
                perms.set_mode(0o444);
                let _ = fs::set_permissions(&*path_str, perms);
            }
        }
    }
}

fn applyfreqbalance() {
    if Path::new("/proc/ppm").exists() {
        dsetfreqppm();
    } else {
        dsetfreq();
    }
}

fn applyfreqgame() {
    if Path::new("/proc/ppm").exists() {
        dsetgamefreqppm();
    } else {
        dsetgamefreq();
    }
}

fn get_biggest_cluster() -> String {
    let mut max_freq = 0;
    let mut target = String::new();

    if let Ok(entries) = glob::glob("/sys/devices/system/cpu/cpufreq/policy*") {
        for entry in entries.flatten() {
            let path_str = entry.to_string_lossy();
            let p = entry.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();

            let cur_freq_str = fs::read_to_string(format!("{}/cpuinfo_max_freq", path_str)).unwrap_or_default().trim().to_string();
            let cur_freq = cur_freq_str.parse::<i64>().unwrap_or(0);

            if cur_freq > max_freq {
                max_freq = cur_freq;
                target = p;
            }
        }
    }
    target
}

fn mediatek_balance() {
    if Path::new("/proc/ppm").exists() && Path::new("/proc/ppm/policy_status").exists() {
        if let Ok(content) = fs::read_to_string("/proc/ppm/policy_status") {
            for line in content.lines() {
                if line.contains("FORCE_LIMIT") || line.contains("PWR_THRO") || line.contains("THERMAL") || line.contains("USER_LIMIT") {
                    if let Some(start) = line.find('[') {
                        if let Some(end) = line[start..].find(']') {
                            let idx = &line[start + 1..start + end];
                            zeshia(&format!("{} 1", idx), "/proc/ppm/policy_status", true);
                        }
                    }
                }
                if line.contains("SYS_BOOST") {
                    if let Some(start) = line.find('[') {
                        if let Some(end) = line[start..].find(']') {
                            let idx = &line[start + 1..start + end];
                            zeshia(&format!("{} 0", idx), "/proc/ppm/policy_status", true);
                        }
                    }
                }
            }
        }
    }

    zeshia("0", "/proc/cpufreq/cpufreq_cci_mode", true);
    zeshia("1", "/proc/cpufreq/cpufreq_power_mode", true);

    if Path::new("/proc/gpufreq").exists() {
        zeshia("0", "/proc/gpufreq/gpufreq_opp_freq", true);
    } else if Path::new("/proc/gpufreqv2").exists() {
        zeshia("-1", "/proc/gpufreqv2/fix_target_opp_index", true);
    }

    zeshia("1", "/sys/devices/system/cpu/eas/enable", true);

    if Path::new("/proc/gpufreq/gpufreq_power_limited").exists() {
        for setting in ["ignore_batt_oc", "ignore_batt_percent", "ignore_low_batt", "ignore_thermal_protect", "ignore_pbm_limited"].iter() {
            zeshia(&format!("{} 0", setting), "/proc/gpufreq/gpufreq_power_limited", true);
        }
    }

    zeshia("0", "/proc/perfmgr/syslimiter/syslimiter_force_disable", true);
    zeshia("stop 0", "/proc/mtk_batoc_throttling/battery_oc_protect_stop", true);
    zeshia("stop 0", "/proc/pbm/pbm_stop", true);
    zeshia("1", "/sys/kernel/eara_thermal/enable", true);

    zeshia("-1", "/sys/devices/platform/10012000.dvfsrc/helio-dvfsrc/dvfsrc_req_ddr_opp", true);
    zeshia("-1", "/sys/kernel/helio-dvfsrc/dvfsrc_force_vcore_dvfs_opp", true);
    zeshia("userspace", "/sys/class/devfreq/mtk-dvfsrc-devfreq/governor", true);
    zeshia("userspace", "/sys/devices/platform/soc/1c00f000.dvfsrc/mtk-dvfsrc-devfreq/devfreq/mtk-dvfsrc-devfreq/governor", true);

    if let Ok(mut entries) = glob::glob("/sys/devices/platform/*.mali") {
        if let Some(Ok(mali_sysfs)) = entries.next() {
            zeshia("coarse_demand", &format!("{}/power_policy", mali_sysfs.to_string_lossy()), true);
        }
    }
}

fn mediatek_performance() {
    if Path::new("/proc/ppm").exists() && Path::new("/proc/ppm/policy_status").exists() {
        if let Ok(content) = fs::read_to_string("/proc/ppm/policy_status") {
            for line in content.lines() {
                if line.contains("FORCE_LIMIT") || line.contains("PWR_THRO") || line.contains("THERMAL") || line.contains("USER_LIMIT") {
                    if let Some(start) = line.find('[') {
                        if let Some(end) = line[start..].find(']') {
                            let idx = &line[start + 1..start + end];
                            zeshia(&format!("{} 0", idx), "/proc/ppm/policy_status", true);
                        }
                    }
                }
                if line.contains("SYS_BOOST") {
                    if let Some(start) = line.find('[') {
                        if let Some(end) = line[start..].find(']') {
                            let idx = &line[start + 1..start + end];
                            zeshia(&format!("{} 1", idx), "/proc/ppm/policy_status", true);
                        }
                    }
                }
            }
        }
    }

    zeshia("1", "/proc/cpufreq/cpufreq_cci_mode", true);
    zeshia("3", "/proc/cpufreq/cpufreq_power_mode", true);

    if Path::new("/proc/gpufreq").exists() {
        if let Ok(content) = fs::read_to_string("/proc/gpufreq/gpufreq_opp_dump") {
            let mut freqs = Vec::new();
            for line in content.lines() {
                if let Some(pos) = line.find("freq = ") {
                    let val_str: String = line[pos+7..].chars().take_while(|c| c.is_digit(10)).collect();
                    if let Ok(val) = val_str.parse::<i64>() {
                        freqs.push(val);
                    }
                }
            }
            if let Some(max) = freqs.iter().max() {
                zeshia(&max.to_string(), "/proc/gpufreq/gpufreq_opp_freq", true);
            }
        }
    } else if Path::new("/proc/gpufreqv2").exists() {
        zeshia("0", "/proc/gpufreqv2/fix_target_opp_index", true);
    }

    zeshia("0", "/sys/devices/system/cpu/eas/enable", true);

    if Path::new("/proc/gpufreq/gpufreq_power_limited").exists() {
        for setting in ["ignore_batt_oc", "ignore_batt_percent", "ignore_low_batt", "ignore_thermal_protect", "ignore_pbm_limited"].iter() {
            zeshia(&format!("{} 1", setting), "/proc/gpufreq/gpufreq_power_limited", true);
        }
    }

    zeshia("0", "/proc/perfmgr/syslimiter/syslimiter_force_disable", true);
    zeshia("stop 1", "/proc/mtk_batoc_throttling/battery_oc_protect_stop", true);
    zeshia("0", "/sys/kernel/eara_thermal/enable", true);

    zeshia("0", "/sys/devices/platform/10012000.dvfsrc/helio-dvfsrc/dvfsrc_req_ddr_opp", true);
    zeshia("0", "/sys/kernel/helio-dvfsrc/dvfsrc_force_vcore_dvfs_opp", true);
    zeshia("performance", "/sys/class/devfreq/mtk-dvfsrc-devfreq/governor", true);
    zeshia("performance", "/sys/devices/platform/soc/1c00f000.dvfsrc/mtk-dvfsrc-devfreq/devfreq/mtk-dvfsrc-devfreq/governor", true);

    if let Ok(mut entries) = glob::glob("/sys/devices/platform/*.mali") {
        if let Some(Ok(mali_sysfs)) = entries.next() {
            zeshia("always_on", &format!("{}/power_policy", mali_sysfs.to_string_lossy()), true);
        }
    }
}

fn mediatek_powersave() {
    if Path::new("/proc/ppm").exists() && Path::new("/proc/ppm/policy_status").exists() {
        if let Ok(content) = fs::read_to_string("/proc/ppm/policy_status") {
            for line in content.lines() {
                if line.contains("FORCE_LIMIT") || line.contains("PWR_THRO") || line.contains("THERMAL") || line.contains("USER_LIMIT") {
                    if let Some(start) = line.find('[') {
                        if let Some(end) = line[start..].find(']') {
                            let idx = &line[start + 1..start + end];
                            zeshia(&format!("{} 1", idx), "/proc/ppm/policy_status", true);
                        }
                    }
                }
                if line.contains("SYS_BOOST") {
                    if let Some(start) = line.find('[') {
                        if let Some(end) = line[start..].find(']') {
                            let idx = &line[start + 1..start + end];
                            zeshia(&format!("{} 0", idx), "/proc/ppm/policy_status", true);
                        }
                    }
                }
            }
        }
    }

    zeshia("0", "/sys/devices/platform/10012000.dvfsrc/helio-dvfsrc/dvfsrc_req_ddr_opp", true);
    zeshia("0", "/sys/kernel/helio-dvfsrc/dvfsrc_force_vcore_dvfs_opp", true);
    zeshia("powersave", "/sys/class/devfreq/mtk-dvfsrc-devfreq/governor", true);
    zeshia("powersave", "/sys/devices/platform/soc/1c00f000.dvfsrc/mtk-dvfsrc-devfreq/devfreq/mtk-dvfsrc-devfreq/governor", true);

    if Path::new("/proc/gpufreq/gpufreq_power_limited").exists() {
        for setting in ["ignore_batt_oc", "ignore_batt_percent", "ignore_low_batt", "ignore_thermal_protect", "ignore_pbm_limited"].iter() {
            zeshia(&format!("{} 1", setting), "/proc/gpufreq/gpufreq_power_limited", true);
        }
    }

    zeshia("0", "/proc/perfmgr/syslimiter/syslimiter_force_disable", true);
    zeshia("stop 0", "/proc/mtk_batoc_throttling/battery_oc_protect_stop", true);
    zeshia("stop 0", "/proc/pbm/pbm_stop", true);
    zeshia("1", "/sys/kernel/eara_thermal/enable", true);

    if let Ok(mut entries) = glob::glob("/sys/devices/platform/*.mali") {
        if let Some(Ok(mali_sysfs)) = entries.next() {
            zeshia("coarse_demand", &format!("{}/power_policy", mali_sysfs.to_string_lossy()), true);
        }
    }
}

fn snapdragon_balance() {
    if let Ok(entries) = glob::glob("/sys/class/devfreq/*cpu-ddr-latfloor*") {
        for entry in entries.flatten() {
            zeshia("compute", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }
    if let Ok(entries) = glob::glob("/sys/class/devfreq/*cpu*-lat") {
        for entry in entries.flatten() {
            zeshia("mem_latency", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }
    if let Ok(entries) = glob::glob("/sys/class/devfreq/*cpu-cpu-ddr-bw") {
        for entry in entries.flatten() {
            zeshia("bw_hwmon", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }
    if let Ok(entries) = glob::glob("/sys/class/devfreq/*cpu-cpu-llcc-bw") {
        for entry in entries.flatten() {
            zeshia("bw_hwmon", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }

    if Path::new("/sys/devices/system/cpu/bus_dcvs/LLCC").exists() {
        let avail = "/sys/devices/system/cpu/bus_dcvs/LLCC/available_frequencies";
        let max_freq = which_maxfreq(avail);
        let min_freq = which_minfreq(avail);
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/LLCC/*/max_freq") {
            for entry in entries.flatten() {
                zeshia(&max_freq, &entry.to_string_lossy(), true);
            }
        }
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/LLCC/*/min_freq") {
            for entry in entries.flatten() {
                zeshia(&min_freq, &entry.to_string_lossy(), true);
            }
        }
    }

    if Path::new("/sys/devices/system/cpu/bus_dcvs/L3").exists() {
        let avail = "/sys/devices/system/cpu/bus_dcvs/L3/available_frequencies";
        let max_freq = which_maxfreq(avail);
        let min_freq = which_minfreq(avail);
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/L3/*/max_freq") {
            for entry in entries.flatten() {
                zeshia(&max_freq, &entry.to_string_lossy(), true);
            }
        }
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/L3/*/min_freq") {
            for entry in entries.flatten() {
                zeshia(&min_freq, &entry.to_string_lossy(), true);
            }
        }
    }

    if Path::new("/sys/devices/system/cpu/bus_dcvs/DDR").exists() {
        let avail = "/sys/devices/system/cpu/bus_dcvs/DDR/available_frequencies";
        let max_freq = which_maxfreq(avail);
        let min_freq = which_minfreq(avail);
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/DDR/*/max_freq") {
            for entry in entries.flatten() {
                zeshia(&max_freq, &entry.to_string_lossy(), true);
            }
        }
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/DDR/*/min_freq") {
            for entry in entries.flatten() {
                zeshia(&min_freq, &entry.to_string_lossy(), true);
            }
        }
    }

    if Path::new("/sys/devices/system/cpu/bus_dcvs/DDRQOS").exists() {
        let avail = "/sys/devices/system/cpu/bus_dcvs/DDRQOS/available_frequencies";
        let max_freq = which_maxfreq(avail);
        let min_freq = which_minfreq(avail);
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/DDRQOS/*/max_freq") {
            for entry in entries.flatten() {
                zeshia(&max_freq, &entry.to_string_lossy(), true);
            }
        }
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/DDRQOS/*/min_freq") {
            for entry in entries.flatten() {
                zeshia(&min_freq, &entry.to_string_lossy(), true);
            }
        }
    }

    let gpu_path = "/sys/class/kgsl/kgsl-3d0/devfreq";
    if Path::new(gpu_path).exists() {
        let avail = format!("{}/available_frequencies", gpu_path);
        let max_freq = which_maxfreq(&avail);
        let freqs = get_freqs(&avail);
        let min_freq = if freqs.len() >= 2 { freqs[1].to_string() } else { which_minfreq(&avail) };
        zeshia(&min_freq, &format!("{}/min_freq", gpu_path), true);
        zeshia(&max_freq, &format!("{}/max_freq", gpu_path), true);
    }

    if let Ok(entries) = glob::glob("/sys/class/devfreq/*gpubw*") {
        for entry in entries.flatten() {
            zeshia("bw_vbif", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }

    zeshia("1", "/sys/class/kgsl/kgsl-3d0/devfreq/adrenoboost", true);
}

fn snapdragon_performance() {
    if let Ok(entries) = glob::glob("/sys/class/devfreq/*cpu-ddr-latfloor*") {
        for entry in entries.flatten() {
            zeshia("performance", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }
    if let Ok(entries) = glob::glob("/sys/class/devfreq/*cpu*-lat") {
        for entry in entries.flatten() {
            zeshia("performance", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }
    if let Ok(entries) = glob::glob("/sys/class/devfreq/*cpu-cpu-ddr-bw") {
        for entry in entries.flatten() {
            zeshia("performance", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }
    if let Ok(entries) = glob::glob("/sys/class/devfreq/*cpu-cpu-llcc-bw") {
        for entry in entries.flatten() {
            zeshia("performance", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }

    if Path::new("/sys/devices/system/cpu/bus_dcvs/LLCC").exists() {
        let avail = "/sys/devices/system/cpu/bus_dcvs/LLCC/available_frequencies";
        let freq = which_maxfreq(avail);
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/LLCC/*/max_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/LLCC/*/min_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
    }

    if Path::new("/sys/devices/system/cpu/bus_dcvs/L3").exists() {
        let avail = "/sys/devices/system/cpu/bus_dcvs/L3/available_frequencies";
        let freq = which_maxfreq(avail);
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/L3/*/max_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/L3/*/min_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
    }

    if Path::new("/sys/devices/system/cpu/bus_dcvs/DDR").exists() {
        let avail = "/sys/devices/system/cpu/bus_dcvs/DDR/available_frequencies";
        let freq = which_maxfreq(avail);
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/DDR/*/max_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/DDR/*/min_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
    }

    if Path::new("/sys/devices/system/cpu/bus_dcvs/DDRQOS").exists() {
        let avail = "/sys/devices/system/cpu/bus_dcvs/DDRQOS/available_frequencies";
        let freq = which_maxfreq(avail);
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/DDRQOS/*/max_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/DDRQOS/*/min_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
    }

    let gpu_path = "/sys/class/kgsl/kgsl-3d0/devfreq";
    if Path::new(gpu_path).exists() {
        let avail = format!("{}/available_frequencies", gpu_path);
        let freq = which_maxfreq(&avail);
        zeshia(&freq, &format!("{}/min_freq", gpu_path), true);
        zeshia(&freq, &format!("{}/max_freq", gpu_path), true);
    }

    if let Ok(entries) = glob::glob("/sys/class/devfreq/*gpubw*") {
        for entry in entries.flatten() {
            zeshia("performance", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }

    zeshia("3", "/sys/class/kgsl/kgsl-3d0/devfreq/adrenoboost", true);
}

fn snapdragon_powersave() {
    if let Ok(entries) = glob::glob("/sys/class/devfreq/*cpu-ddr-latfloor*") {
        for entry in entries.flatten() {
            zeshia("powersave", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }
    if let Ok(entries) = glob::glob("/sys/class/devfreq/*cpu*-lat") {
        for entry in entries.flatten() {
            zeshia("powersave", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }
    if let Ok(entries) = glob::glob("/sys/class/devfreq/*cpu-cpu-ddr-bw") {
        for entry in entries.flatten() {
            zeshia("powersave", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }
    if let Ok(entries) = glob::glob("/sys/class/devfreq/*cpu-cpu-llcc-bw") {
        for entry in entries.flatten() {
            zeshia("powersave", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }

    if Path::new("/sys/devices/system/cpu/bus_dcvs/LLCC").exists() {
        let avail = "/sys/devices/system/cpu/bus_dcvs/LLCC/available_frequencies";
        let freq = which_minfreq(avail);
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/LLCC/*/max_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/LLCC/*/min_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
    }

    if Path::new("/sys/devices/system/cpu/bus_dcvs/L3").exists() {
        let avail = "/sys/devices/system/cpu/bus_dcvs/L3/available_frequencies";
        let freq = which_minfreq(avail);
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/L3/*/max_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/L3/*/min_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
    }

    if Path::new("/sys/devices/system/cpu/bus_dcvs/DDR").exists() {
        let avail = "/sys/devices/system/cpu/bus_dcvs/DDR/available_frequencies";
        let freq = which_minfreq(avail);
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/DDR/*/max_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/DDR/*/min_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
    }

    if Path::new("/sys/devices/system/cpu/bus_dcvs/DDRQOS").exists() {
        let avail = "/sys/devices/system/cpu/bus_dcvs/DDRQOS/available_frequencies";
        let freq = which_minfreq(avail);
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/DDRQOS/*/max_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/bus_dcvs/DDRQOS/*/min_freq") {
            for entry in entries.flatten() {
                zeshia(&freq, &entry.to_string_lossy(), true);
            }
        }
    }

    let gpu_path = "/sys/class/kgsl/kgsl-3d0/devfreq";
    if Path::new(gpu_path).exists() {
        let avail = format!("{}/available_frequencies", gpu_path);
        let freqs = get_freqs(&avail);
        let freq = if freqs.len() >= 2 { freqs[1].to_string() } else { which_minfreq(&avail) };
        zeshia(&freq, &format!("{}/min_freq", gpu_path), true);
        zeshia(&freq, &format!("{}/max_freq", gpu_path), true);
    }

    if let Ok(entries) = glob::glob("/sys/class/devfreq/*gpubw*") {
        for entry in entries.flatten() {
            zeshia("powersave", &format!("{}/governor", entry.to_string_lossy()), true);
        }
    }

    zeshia("0", "/sys/class/kgsl/kgsl-3d0/devfreq/adrenoboost", true);
}

fn exynos_balance() {
    let gpu_path = "/sys/kernel/gpu";
    if Path::new(gpu_path).exists() {
        let avail = format!("{}/gpu_available_frequencies", gpu_path);
        let max_freq = which_maxfreq(&avail);
        let min_freq = which_minfreq(&avail);
        zeshia(&max_freq, &format!("{}/gpu_max_clock", gpu_path), true);
        zeshia(&min_freq, &format!("{}/gpu_min_clock", gpu_path), true);
    }

    if let Ok(mut entries) = glob::glob("/sys/devices/platform/*.mali") {
        if let Some(Ok(mali_sysfs)) = entries.next() {
            zeshia("coarse_demand", &format!("{}/power_policy", mali_sysfs.to_string_lossy()), true);
        }
    }

    let device_mitigation = getprop("persist.sys.azenithconf.devicemitigation").parse::<i64>().unwrap_or(0);
    if device_mitigation == 0 {
        if let Ok(entries) = glob::glob("/sys/class/devfreq/*devfreq_mif*") {
            for entry in entries.flatten() {
                devfreq_unlock(&entry.to_string_lossy());
            }
        }
    }
}

fn exynos_performance() {
    let gpu_path = "/sys/kernel/gpu";
    let lite_mode = getprop("persist.sys.azenithconf.litemode").parse::<i64>().unwrap_or(0);

    if Path::new(gpu_path).exists() {
        let avail = format!("{}/gpu_available_frequencies", gpu_path);
        let max_freq = which_maxfreq(&avail);
        zeshia(&max_freq, &format!("{}/gpu_max_clock", gpu_path), true);

        if lite_mode == 1 {
            let mid_freq = which_midfreq(&avail);
            zeshia(&mid_freq, &format!("{}/gpu_min_clock", gpu_path), true);
        } else {
            zeshia(&max_freq, &format!("{}/gpu_min_clock", gpu_path), true);
        }
    }

    if let Ok(mut entries) = glob::glob("/sys/devices/platform/*.mali") {
        if let Some(Ok(mali_sysfs)) = entries.next() {
            zeshia("always_on", &format!("{}/power_policy", mali_sysfs.to_string_lossy()), true);
        }
    }

    let device_mitigation = getprop("persist.sys.azenithconf.devicemitigation").parse::<i64>().unwrap_or(0);
    if device_mitigation == 0 {
        if let Ok(entries) = glob::glob("/sys/class/devfreq/*devfreq_mif*") {
            for entry in entries.flatten() {
                if lite_mode == 1 {
                    devfreq_mid_perf(&entry.to_string_lossy());
                } else {
                    devfreq_max_perf(&entry.to_string_lossy());
                }
            }
        }
    }
}

fn exynos_powersave() {
    let gpu_path = "/sys/kernel/gpu";
    if Path::new(gpu_path).exists() {
        let avail = format!("{}/gpu_available_frequencies", gpu_path);
        let min_freq = which_minfreq(&avail);
        zeshia(&min_freq, &format!("{}/gpu_min_clock", gpu_path), true);
        zeshia(&min_freq, &format!("{}/gpu_max_clock", gpu_path), true);
    }
}

fn unisoc_balance() {
    if let Ok(mut entries) = glob::glob("/sys/class/devfreq/*.gpu") {
        if let Some(Ok(gpu_path)) = entries.next() {
            devfreq_unlock(&gpu_path.to_string_lossy());
        }
    }
}

fn unisoc_performance() {
    if let Ok(mut entries) = glob::glob("/sys/class/devfreq/*.gpu") {
        if let Some(Ok(gpu_path)) = entries.next() {
            let path_str = gpu_path.to_string_lossy();
            let lite_mode = getprop("persist.sys.azenithconf.litemode").parse::<i64>().unwrap_or(0);
            if lite_mode == 0 {
                devfreq_max_perf(&path_str);
            } else {
                devfreq_mid_perf(&path_str);
            }
        }
    }
}

fn unisoc_powersave() {
    if let Ok(mut entries) = glob::glob("/sys/class/devfreq/*.gpu") {
        if let Some(Ok(gpu_path)) = entries.next() {
            devfreq_min_perf(&gpu_path.to_string_lossy());
        }
    }
}

fn tensor_balance() {
    if let Ok(mut entries) = glob::glob("/sys/devices/platform/*.mali") {
        if let Some(Ok(gpu_path)) = entries.next() {
            let path_str = gpu_path.to_string_lossy();
            let avail = format!("{}/available_frequencies", path_str);
            let max_freq = which_maxfreq(&avail);
            let min_freq = which_minfreq(&avail);
            zeshia(&max_freq, &format!("{}/scaling_max_freq", path_str), true);
            zeshia(&min_freq, &format!("{}/scaling_min_freq", path_str), true);
        }
    }

    let device_mitigation = getprop("persist.sys.azenithconf.devicemitigation").parse::<i64>().unwrap_or(0);
    if device_mitigation == 0 {
        if let Ok(entries) = glob::glob("/sys/class/devfreq/*devfreq_mif*") {
            for entry in entries.flatten() {
                devfreq_unlock(&entry.to_string_lossy());
            }
        }
    }
}

fn tensor_performance() {
    if let Ok(mut entries) = glob::glob("/sys/devices/platform/*.mali") {
        if let Some(Ok(gpu_path)) = entries.next() {
            let path_str = gpu_path.to_string_lossy();
            let avail = format!("{}/available_frequencies", path_str);
            let max_freq = which_maxfreq(&avail);
            zeshia(&max_freq, &format!("{}/scaling_max_freq", path_str), true);

            let lite_mode = getprop("persist.sys.azenithconf.litemode").parse::<i64>().unwrap_or(0);
            if lite_mode == 1 {
                let mid_freq = which_midfreq(&avail);
                zeshia(&mid_freq, &format!("{}/scaling_min_freq", path_str), true);
            } else {
                zeshia(&max_freq, &format!("{}/scaling_min_freq", path_str), true);
            }
        }
    }

    let device_mitigation = getprop("persist.sys.azenithconf.devicemitigation").parse::<i64>().unwrap_or(0);
    if device_mitigation == 0 {
        let lite_mode = getprop("persist.sys.azenithconf.litemode").parse::<i64>().unwrap_or(0);
        if let Ok(entries) = glob::glob("/sys/class/devfreq/*devfreq_mif*") {
            for entry in entries.flatten() {
                if lite_mode == 1 {
                    devfreq_mid_perf(&entry.to_string_lossy());
                } else {
                    devfreq_max_perf(&entry.to_string_lossy());
                }
            }
        }
    }
}

fn tensor_powersave() {
    if let Ok(mut entries) = glob::glob("/sys/devices/platform/*.mali") {
        if let Some(Ok(gpu_path)) = entries.next() {
            let path_str = gpu_path.to_string_lossy();
            let avail = format!("{}/available_frequencies", path_str);
            let freq = which_minfreq(&avail);
            zeshia(&freq, &format!("{}/scaling_min_freq", path_str), true);
            zeshia(&freq, &format!("{}/scaling_max_freq", path_str), true);
        }
    }
}

fn initialize() {
    let params = ["hung_task_timeout_secs", "panic_on_oom", "panic_on_oops", "panic", "softlockup_panic"];
    for param in params.iter() {
        zeshia("0", &format!("/proc/sys/kernel/{}", param), true);
    }

    zeshia("750000", "/proc/sys/kernel/sched_migration_cost_ns", true);
    zeshia("1000000", "/proc/sys/kernel/sched_min_granularity_ns", true);
    zeshia("600000", "/proc/sys/kernel/sched_wakeup_granularity_ns", true);
    zeshia("0", "/proc/sys/vm/page-cluster", true);
    zeshia("20", "/proc/sys/vm/stat_interval", true);
    zeshia("0", "/proc/sys/vm/compaction_proactiveness", true);
    zeshia("255", "/proc/sys/kernel/sched_lib_mask_force", true);

    let _mali_supported = false;
    let mut default_maligov = String::new();
    if let Ok(mut entries) = glob::glob("/sys/devices/platform/soc/*.mali/devfreq/*.mali/governor") {
        if let Some(Ok(mali_gov_path)) = entries.next() {
            let mali_gov = mali_gov_path.to_string_lossy();
            setprop("sys.azenith.maligovsupport", "1");
            let _mali_supported = true;
            if let Ok(content) = fs::read_to_string(&*mali_gov) {
                default_maligov = content.trim().to_string();
                setprop("persist.sys.azenith.default_gpumali_gov", &default_maligov);
                dlog(&format!("Default GPU Mali governor detected: {}", default_maligov));
            }

            let custom_gov = getprop("persist.sys.azenith.custom_default_gpumali_gov");
            if !custom_gov.is_empty() {
                default_maligov = custom_gov;
            }
            dlog(&format!("Using GPU Mali governor: {}", default_maligov));
            zeshia(&default_maligov, &mali_gov, true);

            if getprop("persist.sys.azenith.custom_powersave_gpumali_gov").is_empty() {
                setprop("persist.sys.azenith.custom_powersave_gpumali_gov", &default_maligov);
            }
            if getprop("persist.sys.azenith.custom_performance_gpumali_gov").is_empty() {
                setprop("persist.sys.azenith.custom_performance_gpumali_gov", &default_maligov);
            }
        } else {
            setprop("sys.azenith.maligovsupport", "0");
        }
    } else {
        setprop("sys.azenith.maligovsupport", "0");
    }

    let cpu0_gov_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
    let mut default_cpu_gov = fs::read_to_string(cpu0_gov_path).unwrap_or_default().trim().to_string();
    setprop("persist.sys.azenith.default_cpu_gov", &default_cpu_gov);
    dlog(&format!("Default CPU governor detected: {}", default_cpu_gov));

    if default_cpu_gov == "performance" && getprop("persist.sys.azenith.custom_default_cpu_gov").is_empty() {
        dlog("Default governor is 'performance'");
        let avail_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors";
        if let Ok(content) = fs::read_to_string(avail_path) {
            let govs = ["scx", "schedhorizon", "walt", "sched_pixel", "sugov_ext", "uag", "schedplus", "energy_step", "ondemand", "schedutil", "interactive", "conservative", "powersave"];
            for gov in govs.iter() {
                if content.contains(gov) {
                    setprop("persist.sys.azenith.default_cpu_gov", gov);
                    default_cpu_gov = gov.to_string();
                    dlog(&format!("Fallback governor to: {}", gov));
                    break;
                }
            }
        }
    }

    let custom_cpu_gov = getprop("persist.sys.azenith.custom_default_cpu_gov");
    if !custom_cpu_gov.is_empty() {
        default_cpu_gov = custom_cpu_gov;
    }
    dlog(&format!("Using CPU governor: {}", default_cpu_gov));
    setgov(&default_cpu_gov);
    if getprop("persist.sys.azenith.custom_powersave_cpu_gov").is_empty() {
        setprop("persist.sys.azenith.custom_powersave_cpu_gov", &default_cpu_gov);
    }
    dlog("Parsing CPU Governor complete");

    let mut valid_io = String::new();
    let devs = ["mmcblk0", "mmcblk1", "sda", "sdb", "sdc"];
    for dev in devs.iter() {
        let p = format!("/sys/block/{}/queue/scheduler", dev);
        if Path::new(&p).exists() {
            valid_io = p;
            dlog(&format!("Detected valid block device: {}", dev));
            break;
        }
    }

    if valid_io.is_empty() {
        dlog("No valid block device with scheduler found");
    } else {
        if let Ok(content) = fs::read_to_string(&valid_io) {
            let mut default_io = "none".to_string();
            if let Some(start) = content.find('[') {
                if let Some(end) = content[start..].find(']') {
                    default_io = content[start + 1..start + end].to_string();
                }
            }
            setprop("persist.sys.azenith.default_balanced_IO", &default_io);
            dlog(&format!("Default IO Scheduler detected: {}", default_io));

            let custom_io = getprop("persist.sys.azenith.custom_default_balanced_IO");
            if !custom_io.is_empty() {
                default_io = custom_io;
            }

            sets_io(&default_io);

            if getprop("persist.sys.azenith.custom_powersave_IO").is_empty() {
                setprop("persist.sys.azenith.custom_powersave_IO", &default_io);
            }
            if getprop("persist.sys.azenith.custom_performance_IO").is_empty() {
                setprop("persist.sys.azenith.custom_performance_IO", &default_io);
            }
            dlog("Parsing IO Scheduler complete");
        }
    }

    parse_resolution();
    apply_init_logic();

    if getprop("persist.sys.azenithconf.disabletrace") == "1" {
        dlog("Applying disable trace");
        let traces = ["/sys/kernel/tracing/instances/mmstat/trace", "/sys/kernel/tracing/trace"];
        for t in traces.iter() {
            zeshia("", t, true);
        }
        zeshia("0", "/sys/kernel/tracing/options/overwrite", true);
        zeshia("0", "/sys/kernel/tracing/options/record-tgids", true);
    }

    let logd_state = getprop("persist.sys.azenithconf.logd");
    let loggers = ["logd", "traced", "statsd", "tcpdump", "cnss_diag", "subsystem_ramdump", "charge_logger", "wlan_logging"];
    if logd_state == "1" {
        for l in loggers.iter() {
            let _ = Command::new("stop").arg(l).output();
        }
        dlog("Applying Kill Logd");
    } else {
        for l in loggers.iter() {
            let _ = Command::new("start").arg(l).output();
        }
    }

    let vsync_value = getprop("persist.sys.azenithconf.vsync");
    let _ = Command::new("sys.azenith-utilityconf").arg("disablevsync").arg(&vsync_value).output();

    let _ = Command::new("sys.azenith-utilityconf").arg("checkBypass").output();
    let _ = Command::new("sync").output();

    az_log("Initializing Complete");
    dlog("Initializing Complete");
}

fn performance_profile() {
    let _default_cpu_gov = {
        let custom = getprop("persist.sys.azenith.custom_default_cpu_gov");
        if !custom.is_empty() { custom }
        else {
            let def = getprop("persist.sys.azenith.default_cpu_gov");
            if !def.is_empty() { def } else { "performance".to_string() }
        }
    };

    let litemode = getprop("persist.sys.azenithconf.litemode").parse::<i64>().unwrap_or(0);
    if litemode == 0 {
        setgov("performance");
        dlog("Applying global governor: performance");
    } else {
        let big_policy = get_biggest_cluster();
        if !big_policy.is_empty() {
            zeshia("performance", &format!("/sys/devices/system/cpu/cpufreq/{}/scaling_governor", big_policy), true);
            dlog(&format!("Applying performance only to biggest cluster: {}", big_policy));
        }
    }

    let default_iosched = {
        let custom = getprop("persist.sys.azenith.custom_default_balanced_IO");
        if !custom.is_empty() { custom }
        else {
            let def = getprop("persist.sys.azenith.default_balanced_IO");
            if !def.is_empty() { def } else { "none".to_string() }
        }
    };

    let custom_perf_io = getprop("persist.sys.azenith.custom_performance_IO");
    if !custom_perf_io.is_empty() {
        sets_io(&custom_perf_io);
        dlog(&format!("Applying I/O scheduler to : {}", custom_perf_io));
    } else {
        sets_io(&default_iosched);
        dlog(&format!("Applying I/O scheduler to : {}", default_iosched));
    }

    let mali_comp = getprop("sys.azenith.maligovsupport").parse::<i64>().unwrap_or(0);
    if mali_comp == 1 {
        let custom_maligov = getprop("persist.sys.azenith.custom_performance_gpumali_gov");
        if !custom_maligov.is_empty() {
            sets_gpu_mali(&custom_maligov);
            dlog(&format!("Applying GPU Mali Governor to : {}", custom_maligov));
        } else {
            let def = getprop("persist.sys.azenith.default_gpumali_gov");
            let m_gov = if !def.is_empty() { def } else { "dummy".to_string() };
            sets_gpu_mali(&m_gov);
            dlog(&format!("Applying GPU Mali Governor to : {}", m_gov));
        }
    }

    if getprop("persist.sys.azenithconf.bypasschg") == "1" {
        let _ = Command::new("sys.azenith-utilityconf").arg("enableBypass").output();
    }

    if Path::new("/proc/ppm").exists() {
        setgamefreqppm();
    } else {
        setgamefreq();
    }

    if litemode == 0 {
        dlog("Set CPU freq to max available Frequencies");
    } else {
        dlog("Set CPU freq to normal Frequencies");
    }

    if let Ok(entries) = glob::glob("/sys/devices/system/cpu/perf/*") {
        for entry in entries.flatten() {
            let path_str = entry.to_string_lossy();
            if path_str.ends_with("gpu_pmu_enable") || path_str.ends_with("fuel_gauge_enable") || path_str.ends_with("enable") || path_str.ends_with("charger_enable") {
                zeshia("1", &path_str, true);
            }
        }
    }

    zeshia("40", "/proc/sys/vm/vfs_cache_pressure", true);
    zeshia("3", "/proc/sys/vm/drop_caches", true);

    zeshia("N", "/sys/module/workqueue/parameters/power_efficient", true);
    zeshia("N", "/sys/module/workqueue/parameters/disable_numa", true);
    zeshia("0", "/sys/kernel/eara_thermal/enable", true);
    zeshia("0", "/sys/devices/system/cpu/eas/enable", true);
    zeshia("1", "/sys/devices/system/cpu/cpu2/online", true);
    zeshia("1", "/sys/devices/system/cpu/cpu3/online", true);

    apply_stune_boost(true, false);

    zeshia("1", "/proc/sys/kernel/perf_cpu_time_max_percent", true);
    zeshia("1", "/proc/sys/kernel/sched_energy_aware", true);

    apply_core_ctl("0");
    apply_battery_saver("0");

    zeshia("0", "/proc/sys/kernel/split_lock_mitigate", true);
    apply_sched_features(&["NEXT_BUDDY", "NO_TTWU_QUEUE"]);

    if getprop("persist.sys.azenithconf.clearbg") == "1" {
        clear_background_apps();
    }

    if litemode == 0 {
        match getprop("persist.sys.azenithdebug.soctype").as_str() {
            "1" => mediatek_performance(),
            "2" => snapdragon_performance(),
            "3" => exynos_performance(),
            "4" => unisoc_performance(),
            "5" => tensor_performance(),
            _ => {}
        }
    }
    az_log("Performance Profile Applied Successfully!");
}

fn balanced_profile() {
    let default_cpu_gov = {
        let custom = getprop("persist.sys.azenith.custom_default_cpu_gov");
        if !custom.is_empty() { custom }
        else {
            let def = getprop("persist.sys.azenith.default_cpu_gov");
            if !def.is_empty() { def } else { "schedutil".to_string() }
        }
    };
    setgov(&default_cpu_gov);
    dlog(&format!("Applying governor to : {}", default_cpu_gov));

    let default_iosched = {
        let custom = getprop("persist.sys.azenith.custom_default_balanced_IO");
        if !custom.is_empty() { custom }
        else {
            let def = getprop("persist.sys.azenith.default_balanced_IO");
            if !def.is_empty() { def } else { "none".to_string() }
        }
    };
    sets_io(&default_iosched);
    dlog(&format!("Applying I/O scheduler to : {}", default_iosched));

    let mali_comp = getprop("sys.azenith.maligovsupport").parse::<i64>().unwrap_or(0);
    if mali_comp == 1 {
        let default_maligov = {
            let custom = getprop("persist.sys.azenith.custom_default_gpumali_gov");
            if !custom.is_empty() { custom }
            else {
                let def = getprop("persist.sys.azenith.default_gpumali_gov");
                if !def.is_empty() { def } else { "dummy".to_string() }
            }
        };
        sets_gpu_mali(&default_maligov);
        dlog(&format!("Applying GPU Mali Governor to : {}", default_maligov));
    }

    if getprop("persist.sys.azenithconf.bypasschg") == "1" {
        let _ = Command::new("sys.azenith-utilityconf").arg("disableBypass").output();
    }

    if Path::new("/proc/ppm").exists() {
        setfreqppm();
    } else {
        setfreq();
    }

    if getprop("persist.sys.azenithconf.freqoffset") == "Disabled" {
        dlog("Set CPU freq to normal Frequencies");
    } else {
        dlog("Set CPU freq to normal selected Frequencies");
    }

    for pl in glob::glob("/sys/devices/system/cpu/perf/*").unwrap().flatten() {
        let p = pl.to_string_lossy();
        if p.ends_with("gpu_pmu_enable") || p.ends_with("fuel_gauge_enable") || p.ends_with("enable") {
            zeshia("0", &p, true);
        }
        if p.ends_with("charger_enable") {
            zeshia("1", &p, true);
        }
    }

    zeshia("120", "/proc/sys/vm/vfs_cache_pressure", true);
    zeshia("Y", "/sys/module/workqueue/parameters/power_efficient", true);
    zeshia("Y", "/sys/module/workqueue/parameters/disable_numa", true);
    zeshia("1", "/sys/kernel/eara_thermal/enable", true);
    zeshia("1", "/sys/devices/system/cpu/eas/enable", true);

    apply_stune_boost(false, false);

    zeshia("100", "/proc/sys/kernel/perf_cpu_time_max_percent", true);
    zeshia("2", "/proc/sys/kernel/perf_cpu_time_max_percent", true);
    zeshia("1", "/proc/sys/kernel/sched_energy_aware", true);

    apply_core_ctl("0");
    apply_battery_saver("0");

    zeshia("1", "/proc/sys/kernel/split_lock_mitigate", true);
    apply_sched_features(&["NEXT_BUDDY", "TTWU_QUEUE"]);

    match getprop("persist.sys.azenithdebug.soctype").as_str() {
        "1" => mediatek_balance(),
        "2" => snapdragon_balance(),
        "3" => exynos_balance(),
        "4" => unisoc_balance(),
        "5" => tensor_balance(),
        _ => {}
    }

    az_log("Balanced Profile applied successfully!");
}

fn eco_mode() {
    let powersave_cpu_gov = {
        let custom = getprop("persist.sys.azenith.custom_powersave_cpu_gov");
        if !custom.is_empty() { custom } else { "powersave".to_string() }
    };
    setgov(&powersave_cpu_gov);
    dlog(&format!("Applying governor to : {}", powersave_cpu_gov));

    let powersave_iosched = {
        let custom = getprop("persist.sys.azenith.custom_powersave_IO");
        if !custom.is_empty() { custom } else { "none".to_string() }
    };
    sets_io(&powersave_iosched);
    dlog(&format!("Applying I/O scheduler to : {}", powersave_iosched));

    let mali_comp = getprop("sys.azenith.maligovsupport").parse::<i64>().unwrap_or(0);
    if mali_comp == 1 {
        let powersave_maligov = {
            let custom = getprop("persist.sys.azenith.custom_powersave_gpumali_gov");
            if !custom.is_empty() { custom } else { "dummy".to_string() }
        };
        sets_gpu_mali(&powersave_maligov);
        dlog(&format!("Applying GPU Mali Governor to : {}", powersave_maligov));
    }

    if Path::new("/proc/ppm").exists() {
        setfreqppm();
    } else {
        setfreq();
    }
    dlog("Set CPU freq to low Frequencies");

    if getprop("persist.sys.azenithconf.bypasschg") == "1" {
        let _ = Command::new("sys.azenith-utilityconf").arg("disableBypass").output();
    }

    for pl in glob::glob("/sys/devices/system/cpu/perf/*").unwrap().flatten() {
        let p = pl.to_string_lossy();
        if p.ends_with("gpu_pmu_enable") || p.ends_with("fuel_gauge_enable") || p.ends_with("enable") {
            zeshia("0", &p, true);
        }
        if p.ends_with("charger_enable") {
            zeshia("1", &p, true);
        }
    }

    zeshia("120", "/proc/sys/vm/vfs_cache_pressure", true);
    zeshia("Y", "/sys/module/workqueue/parameters/power_efficient", true);
    zeshia("Y", "/sys/module/workqueue/parameters/disable_numa", true);
    zeshia("1", "/sys/kernel/eara_thermal/enable", true);
    zeshia("1", "/sys/devices/system/cpu/eas/enable", true);

    apply_stune_boost(false, true);

    zeshia("50", "/proc/sys/kernel/perf_cpu_time_max_percent", true);
    zeshia("0", "/proc/sys/kernel/perf_cpu_time_max_percent", true);
    zeshia("0", "/proc/sys/kernel/sched_energy_aware", true);

    apply_core_ctl("0");
    apply_battery_saver("1");

    zeshia("1", "/proc/sys/kernel/split_lock_mitigate", true);
    apply_sched_features(&["NO_NEXT_BUDDY", "NO_TTWU_QUEUE"]);

    match getprop("persist.sys.azenithdebug.soctype").as_str() {
        "1" => mediatek_powersave(),
        "2" => snapdragon_powersave(),
        "3" => exynos_powersave(),
        "4" => unisoc_powersave(),
        "5" => tensor_powersave(),
        _ => {}
    }

    az_log("ECO Mode applied successfully!");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let command = &args[1];
        match command.as_str() {
            "0" => initialize(),
            "1" => performance_profile(),
            "2" => balanced_profile(),
            "3" => eco_mode(),
            _ => {
                // Ignore other args
            }
        }
    }
}
fn apply_init_logic() {
    let justintime_state = getprop("persist.sys.azenithconf.justintime").parse::<i64>().unwrap_or(0);
    if justintime_state == 1 {
        dlog("Applying JIT Compiler");
        if let Ok(out) = Command::new("cmd").args(&["package", "list", "packages", "-3"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
                if let Some(pos) = line.find(':') {
                    let pkg = &line[pos + 1..];
                    let _ = Command::new("cmd").args(&["package", "compile", "-m", "speed-profile", pkg]).spawn();
                    az_log(&format!("{} | Success", pkg));
                }
            }
        }
    }

    let schedtunes_state = getprop("persist.sys.azenithconf.schedtunes").parse::<i64>().unwrap_or(0);
    if schedtunes_state == 1 {
        dlog("Applying Schedtunes for Schedutil and Schedhorizon");
        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/cpufreq/policy*") {
            for entry in entries.flatten() {
                let policy_path = entry.to_string_lossy();
                let freqs_path = format!("{}/scaling_available_frequencies", policy_path);
                if let Ok(content) = fs::read_to_string(&freqs_path) {
                    let mut freqs: Vec<i64> = content.split_whitespace().filter_map(|s| s.parse().ok()).collect();
                    freqs.sort_by(|a, b| b.cmp(a));
                    let selected_freqs: Vec<String> = freqs.into_iter().take(6).map(|f| f.to_string()).collect();
                    let selected_str = selected_freqs.join(" ");
                    let num = selected_freqs.len();

                    let mut up_delay = String::new();
                    for i in 1..=num {
                        up_delay.push_str(&format!("{} ", 50 * i));
                    }
                    let up_delay = up_delay.trim();

                    let up_rate = "6500";
                    let down_rate = "12000";
                    let rate_limit = "7000";

                    let schedhorizon = format!("{}/schedhorizon", policy_path);
                    let schedutil = format!("{}/schedutil", policy_path);

                    if Path::new(&schedhorizon).exists() {
                        zeshia(up_delay, &format!("{}/up_delay", schedhorizon), true);
                        zeshia(&selected_str, &format!("{}/efficient_freq", schedhorizon), true);
                        if Path::new(&format!("{}/up_rate_limit_us", schedhorizon)).exists() {
                            zeshia(up_rate, &format!("{}/up_rate_limit_us", schedhorizon), true);
                        } else if Path::new(&format!("{}/rate_limit_us", schedhorizon)).exists() {
                            zeshia(rate_limit, &format!("{}/rate_limit_us", schedhorizon), true);
                        }
                        zeshia(down_rate, &format!("{}/down_rate_limit_us", schedhorizon), true);
                    }

                    if Path::new(&schedutil).exists() {
                        if Path::new(&format!("{}/up_rate_limit_us", schedutil)).exists() {
                            zeshia(up_rate, &format!("{}/up_rate_limit_us", schedutil), true);
                        } else if Path::new(&format!("{}/rate_limit_us", schedutil)).exists() {
                            zeshia(rate_limit, &format!("{}/rate_limit_us", schedutil), true);
                        }
                        zeshia(down_rate, &format!("{}/down_rate_limit_us", schedutil), true);
                    }
                }
            }
        }
    }

    let walt_state = getprop("persist.sys.azenithconf.walttunes").parse::<i64>().unwrap_or(0);
    if walt_state == 1 {
        dlog("Applying WALT governor tuning");
        let walt_up_rate = "8000";
        let walt_down_rate = "12000";
        let walt_hispeed_load = "92";
        let walt_top_freq_count = 6;
        let walt_target_start = 95;
        let walt_target_step = 8;

        if let Ok(entries) = glob::glob("/sys/devices/system/cpu/cpufreq/policy*") {
            for entry in entries.flatten() {
                let policy_path = entry.to_string_lossy();
                let walt_path = format!("{}/walt", policy_path);
                if !Path::new(&walt_path).exists() { continue; }

                if let Ok(content) = fs::read_to_string(format!("{}/scaling_available_frequencies", policy_path)) {
                    let mut freqs: Vec<i64> = content.split_whitespace().filter_map(|s| s.parse().ok()).collect();
                    freqs.sort_by(|a, b| b.cmp(a));
                    let selected_freqs: Vec<String> = freqs.into_iter().take(walt_top_freq_count).map(|f| f.to_string()).collect();
                    if selected_freqs.is_empty() { continue; }

                    let highest = &selected_freqs[0];
                    let second = if selected_freqs.len() > 1 { &selected_freqs[1] } else { highest };

                    let mut tloads = String::new();
                    let mut cur = walt_target_start;
                    for _ in 0..selected_freqs.len() {
                        tloads.push_str(&format!("{} ", cur));
                        cur -= walt_target_step;
                        if cur < 10 { cur = 10; }
                    }
                    let tloads = tloads.trim();
                    let selected_str = selected_freqs.join(" ");

                    zeshia(walt_hispeed_load, &format!("{}/hispeed_load", walt_path), true);
                    zeshia(second, &format!("{}/hispeed_freq", walt_path), true);
                    zeshia(highest, &format!("{}/rtg_boost_freq", walt_path), true);
                    zeshia(tloads, &format!("{}/target_loads", walt_path), true);
                    zeshia(&selected_str, &format!("{}/efficient_freq", walt_path), true);
                    zeshia(walt_up_rate, &format!("{}/up_rate_limit_us", walt_path), true);
                    zeshia(walt_down_rate, &format!("{}/down_rate_limit_us", walt_path), true);
                }
            }
        }
    }

    let fpsged_state = getprop("persist.sys.azenithconf.fpsged").parse::<i64>().unwrap_or(0);
    if fpsged_state == 1 {
        dlog("Applying FPSGO Parameters");
        let ged_params = [
            ("ged_smart_boost", "1"), ("boost_upper_bound", "100"), ("enable_gpu_boost", "1"),
            ("enable_cpu_boost", "1"), ("ged_boost_enable", "1"), ("boost_gpu_enable", "1"),
            ("gpu_dvfs_enable", "1"), ("gx_frc_mode", "1"), ("gx_dfps", "1"), ("gx_force_cpu_boost", "1"),
            ("gx_boost_on", "1"), ("gx_game_mode", "1"), ("gx_3D_benchmark_on", "1"), ("gpu_loading", "0"),
            ("cpu_boost_policy", "1"), ("boost_extra", "1"), ("is_GED_KPI_enabled", "0")
        ];
        for (param, value) in ged_params.iter() {
            zeshia(value, &format!("/sys/module/ged/parameters/{}", param), true);
        }

        zeshia("0", "/sys/kernel/fpsgo/fbt/boost_ta", true);
        zeshia("1", "/sys/kernel/fpsgo/fbt/enable_switch_down_throttle", true);
        zeshia("1", "/sys/kernel/fpsgo/fstb/adopt_low_fps", true);
        zeshia("1", "/sys/kernel/fpsgo/fstb/fstb_self_ctrl_fps_enable", true);
        zeshia("0", "/sys/kernel/fpsgo/fstb/boost_ta", true);
        zeshia("1", "/sys/kernel/fpsgo/fstb/enable_switch_sync_flag", true);
        zeshia("0", "/sys/kernel/fpsgo/fbt/boost_VIP", true);
        zeshia("1", "/sys/kernel/fpsgo/fstb/gpu_slowdown_check", true);
        zeshia("1", "/sys/kernel/fpsgo/fbt/thrm_limit_cpu", true);
        zeshia("0", "/sys/kernel/fpsgo/fbt/thrm_temp_th", true);
        zeshia("0", "/sys/kernel/fpsgo/fbt/llf_task_policy", true);
        zeshia("100", "/sys/module/mtk_fpsgo/parameters/uboost_enhance_f", true);
        zeshia("0", "/sys/module/mtk_fpsgo/parameters/isolation_limit_cap", true);
        zeshia("1", "/sys/pnpmgr/fpsgo_boost/boost_enable", true);
        zeshia("1", "/sys/pnpmgr/fpsgo_boost/boost_mode", true);
        zeshia("1", "/sys/pnpmgr/install", true);
        zeshia("100", "/sys/kernel/ged/hal/gpu_boost_level", true);
    }

    let malisched_state = getprop("persist.sys.azenithconf.malisched").parse::<i64>().unwrap_or(0);
    if malisched_state == 1 {
        dlog("Applying GPU Mali Sched");
        if let Ok(mut entries) = glob::glob("/sys/devices/platform/soc/*mali*/scheduling") {
            if let Some(Ok(mali_dir)) = entries.next() {
                zeshia("full", &format!("{}/serialize_jobs", mali_dir.to_string_lossy()), true);
            }
        }
        if let Ok(mut entries) = glob::glob("/sys/devices/platform/soc/*mali*") {
            if let Some(Ok(mali1_dir)) = entries.next() {
                zeshia("1", &format!("{}/js_ctx_scheduling_mode", mali1_dir.to_string_lossy()), true);
            }
        }
    }

    let sfl_state = getprop("persist.sys.azenithconf.SFL").parse::<i64>().unwrap_or(0);
    if sfl_state == 1 {
        dlog("Applying SurfaceFlinger Latency");

        let mut samples: Vec<i64> = Vec::new();
        for _ in 0..5 {
            if let Ok(out) = Command::new("dumpsys").args(&["SurfaceFlinger", "--latency"]).output() {
                let s = String::from_utf8_lossy(&out.stdout);
                if let Some(first_line) = s.lines().next() {
                    if let Ok(period) = first_line.trim().parse::<i64>() {
                        if period > 0 {
                            let rate = (1_000_000_000 + (period / 2)) / period;
                            if rate >= 30 && rate <= 240 {
                                samples.push(rate);
                            }
                        }
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let refresh_rate = if samples.is_empty() {
            60
        } else {
            samples.sort_unstable();
            let count = samples.len();
            let mid = count / 2;
            if count % 2 == 1 {
                samples[mid]
            } else {
                (samples[mid - 1] + samples[mid]) / 2
            }
        };

        let frame_duration_ns = 1_000_000_000.0 / (refresh_rate as f64);

        let mut cpu_load = 0.0;
        if let Ok(out) = Command::new("top").args(&["-n", "1", "-b"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
                if line.contains("Cpu(s)") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        let val1 = parts[1].replace("%user", "").replace("us,", "").parse::<f64>().unwrap_or(0.0);
                        let val2 = parts[3].replace("%sys", "").replace("sy,", "").parse::<f64>().unwrap_or(0.0);
                        cpu_load = val1 + val2;
                        break;
                    }
                }
            }
        }

        let base_margin = 0.07;
        let margin_ratio = if cpu_load > 70.0 { base_margin + 0.01 } else { base_margin };
        let min_margin = (frame_duration_ns * margin_ratio).round() as i64;

        let (app_phase_ratio, sf_phase_ratio, app_duration_ratio, sf_duration_ratio) = if refresh_rate >= 120 {
            (0.68, 0.85, 0.58, 0.32)
        } else if refresh_rate >= 90 {
            (0.66, 0.82, 0.60, 0.30)
        } else if refresh_rate >= 75 {
            (0.64, 0.80, 0.62, 0.28)
        } else {
            (0.62, 0.75, 0.65, 0.25)
        };

        let app_phase_offset_ns = (-frame_duration_ns * app_phase_ratio).round() as i64;
        let sf_phase_offset_ns = (-frame_duration_ns * sf_phase_ratio).round() as i64;

        let mut app_duration = (frame_duration_ns * app_duration_ratio).round() as i64;
        let mut sf_duration = (frame_duration_ns * sf_duration_ratio).round() as i64;

        let app_end_time = app_phase_offset_ns + app_duration;
        let dead_time = -(app_end_time + sf_phase_offset_ns);

        if dead_time < min_margin {
            let adjustment = min_margin - dead_time;
            let new_app_duration = app_duration - adjustment;
            app_duration = if new_app_duration > 0 { new_app_duration } else { 0 };
            az_log(&format!("Optimization: Adjusted app duration by -{}ns for dynamic margin", adjustment));
        }

        let min_phase_duration = (frame_duration_ns * 0.12).round() as i64;
        if app_duration < min_phase_duration {
            app_duration = min_phase_duration;
        }
        if sf_duration < min_phase_duration {
            sf_duration = min_phase_duration;
        }

        let _ = Command::new("resetprop").args(&["-n", "debug.sf.early.app.duration", &app_duration.to_string()]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.earlyGl.app.duration", &app_duration.to_string()]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.late.app.duration", &app_duration.to_string()]).output();

        let _ = Command::new("resetprop").args(&["-n", "debug.sf.early.sf.duration", &sf_duration.to_string()]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.earlyGl.sf.duration", &sf_duration.to_string()]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.late.sf.duration", &sf_duration.to_string()]).output();

        let _ = Command::new("resetprop").args(&["-n", "debug.sf.early_app_phase_offset_ns", &app_phase_offset_ns.to_string()]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.high_fps_early_app_phase_offset_ns", &app_phase_offset_ns.to_string()]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.high_fps_late_app_phase_offset_ns", &app_phase_offset_ns.to_string()]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.early_phase_offset_ns", &sf_phase_offset_ns.to_string()]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.high_fps_early_phase_offset_ns", &sf_phase_offset_ns.to_string()]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.high_fps_late_sf_phase_offset_ns", &sf_phase_offset_ns.to_string()]).output();

        let threshold_ratio = if refresh_rate >= 120 {
            0.28
        } else if refresh_rate >= 90 {
            0.32
        } else if refresh_rate >= 75 {
            0.35
        } else {
            0.38
        };

        let mut phase_offset_threshold_ns = (frame_duration_ns * threshold_ratio).round() as i64;
        let max_threshold = (frame_duration_ns * 0.45).round() as i64;
        let min_threshold = (frame_duration_ns * 0.22).round() as i64;

        if phase_offset_threshold_ns > max_threshold {
            phase_offset_threshold_ns = max_threshold;
        } else if phase_offset_threshold_ns < min_threshold {
            phase_offset_threshold_ns = min_threshold;
        }

        let _ = Command::new("resetprop").args(&["-n", "debug.sf.phase_offset_threshold_for_next_vsync_ns", &phase_offset_threshold_ns.to_string()]).output();

        let _ = Command::new("resetprop").args(&["-n", "debug.sf.enable_advanced_sf_phase_offset", "1"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.predict_hwc_composition_strategy", "1"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.use_phase_offsets_as_durations", "1"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.disable_hwc_vds", "1"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.show_refresh_rate_overlay_spinner", "0"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.show_refresh_rate_overlay_render_rate", "0"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.show_refresh_rate_overlay_in_middle", "0"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.kernel_idle_timer_update_overlay", "0"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.dump.enable", "0"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.dump.external", "0"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.dump.primary", "0"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.treat_170m_as_sRGB", "0"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.luma_sampling", "0"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.showupdates", "0"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.disable_client_composition_cache", "0"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.treble_testing_override", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.enable_layer_caching", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.enable_cached_set_render_scheduling", "true"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.layer_history_trace", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.edge_extension_shader", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.enable_egl_image_tracker", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.use_phase_offsets_as_durations", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.layer_caching_highlight", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.enable_hwc_vds", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.vsp_trace", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.sf.enable_transaction_tracing", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.filter_test_overhead", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.show_layers_updates", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.capture_skp_enabled", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.trace_gpu_resources", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.skia_tracing_enabled", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.nv_profiling", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.skia_use_perfetto_track_events", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.show_dirty_regions", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.profile", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.overdraw", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.show_non_rect_clip", "hide"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.webview_overlays_enabled", "false"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.skip_empty_damage", "true"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.use_gpu_pixel_buffers", "true"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.use_buffer_age", "true"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.use_partial_updates", "true"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.skip_eglmanager_telemetry", "true"]).output();
        let _ = Command::new("resetprop").args(&["-n", "debug.hwui.level", "0"]).output();
    }

    let dthermal_state = getprop("persist.sys.azenithconf.DThermal").parse::<i64>().unwrap_or(0);
    if dthermal_state == 1 {
        let _ = Command::new("pkill").arg("-f").arg("thermald").output();

        let out = Command::new("find").args(&["/system/etc/init", "/vendor/etc/init", "/odm/etc/init", "-type", "f"]).output();
        if let Ok(out) = out {
            let files = String::from_utf8_lossy(&out.stdout);
            for file in files.lines() {
                if let Ok(content) = fs::read_to_string(file) {
                    for line in content.lines() {
                        if line.starts_with("service") && line.to_lowercase().contains("thermal") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() > 1 {
                                let _ = Command::new("stop").arg(parts[1]).output();
                            }
                        }
                    }
                }
            }
        }

        let props_out = Command::new("getprop").output();
        if let Ok(out) = props_out {
            let props = String::from_utf8_lossy(&out.stdout);
            for line in props.lines() {
                if line.to_lowercase().contains("thermal") {
                    if let Some(start) = line.find('[') {
                        if let Some(end) = line[start..].find(']') {
                            let prop = &line[start + 1..start + end];
                            let _ = Command::new("resetprop").args(&["-n", prop, "suspended"]).output();
                        }
                    }
                }
            }
        }

        let ps_out = Command::new("ps").arg("-A").output();
        if let Ok(out) = ps_out {
            let ps = String::from_utf8_lossy(&out.stdout);
            for line in ps.lines() {
                let low = line.to_lowercase();
                if low.contains("thermal-engine") || low.contains("thermald") || low.contains("mtk_thermal") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 1 {
                        let _ = Command::new("kill").arg("-9").arg(parts[1]).output();
                    }
                }
            }
        }

        if let Ok(entries) = glob::glob("/sys/class/thermal/thermal_zone*/mode") {
            for entry in entries.flatten() {
                zeshia("disabled", &entry.to_string_lossy(), true);
            }
        }
        if let Ok(entries) = glob::glob("/sys/class/thermal/thermal_zone*/policy") {
            for entry in entries.flatten() {
                zeshia("userspace", &entry.to_string_lossy(), true);
            }
        }

        let gpu_limit = "/proc/gpufreq/gpufreq_power_limited";
        if Path::new(gpu_limit).exists() {
            for k in ["ignore_batt_oc", "ignore_batt_percent", "ignore_low_batt", "ignore_thermal_protect", "ignore_pbm_limited"].iter() {
                zeshia(&format!("{} 1", k), gpu_limit, true);
            }
        }

        let ppm = "/proc/ppm/policy_status";
        if Path::new(ppm).exists() {
            if let Ok(content) = fs::read_to_string(ppm) {
                for line in content.lines() {
                    if line.contains("FORCE_LIMIT") || line.contains("PWR_THRO") || line.contains("THERMAL") {
                        if let Some(start) = line.find('[') {
                            if let Some(end) = line[start..].find(']') {
                                let idx = &line[start + 1..start + end];
                                zeshia(&format!("{} 0", idx), ppm, true);
                            }
                        }
                    }
                }
            }
        }

        if let Ok(entries) = glob::glob("/sys/devices/virtual/thermal/thermal_zone*/temp") {
            for entry in entries.flatten() {
                if let Ok(mut perms) = fs::metadata(&entry).map(|m| m.permissions()) {
                    perms.set_mode(0o000);
                    let _ = fs::set_permissions(&entry, perms);
                }
            }
        }
        if let Ok(entries) = glob::glob("/sys/devices/virtual/thermal/thermal_zone*/trip_point_*") {
            for entry in entries.flatten() {
                if let Ok(mut perms) = fs::metadata(&entry).map(|m| m.permissions()) {
                    perms.set_mode(0o000);
                    let _ = fs::set_permissions(&entry, perms);
                }
            }
        }

        let _ = Command::new("cmd").args(&["thermalservice", "override-status", "0"]).output();

        let batoc = "/proc/mtk_batoc_throttling/battery_oc_protect_stop";
        if Path::new(batoc).exists() {
            zeshia("stop 1", batoc, true);
        }

        az_log("Thermal is disabled");
    }
}
fn apply_stune_boost(is_performance: bool, is_eco: bool) {
    if let Ok(entries) = glob::glob("/dev/stune/*") {
        for entry in entries.flatten() {
            let path_str = entry.to_string_lossy();
            let base = entry.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();

            if is_performance {
                zeshia("30", &format!("{}/schedtune.boost", path_str), true);
                zeshia("1", &format!("{}/schedtune.sched_boost_enabled", path_str), true);
                zeshia("0", &format!("{}/schedtune.prefer_idle", path_str), true);
                zeshia("0", &format!("{}/schedtune.colocate", path_str), true);
            } else if is_eco {
                zeshia("0", &format!("{}/schedtune.boost", path_str), true);
                zeshia("0", &format!("{}/schedtune.sched_boost_enabled", path_str), true);
                zeshia("0", &format!("{}/schedtune.prefer_idle", path_str), true);
                zeshia("0", &format!("{}/schedtune.colocate", path_str), true);
            } else {
                // balanced
                zeshia("0", &format!("{}/schedtune.boost", path_str), true);
                zeshia("0", &format!("{}/schedtune.sched_boost_enabled", path_str), true);
                zeshia("0", &format!("{}/schedtune.prefer_idle", path_str), true);
                zeshia("0", &format!("{}/schedtune.colocate", path_str), true);
            }
        }
    }
}

fn apply_core_ctl(boost: &str) {
    if let Ok(entries) = glob::glob("/sys/devices/system/cpu/cpu*") {
        for entry in entries.flatten() {
            let path_str = entry.to_string_lossy();
            zeshia("0", &format!("{}/core_ctl/enable", path_str), true);
            zeshia(boost, &format!("{}/core_ctl/core_ctl_boost", path_str), true);
        }
    }
}

fn apply_battery_saver(enabled: &str) {
    let path = "/sys/module/battery_saver/parameters/enabled";
    if Path::new(path).exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if content.chars().any(|c| c.is_digit(10)) {
                let val = if enabled == "Y" || enabled == "1" { "1" } else { "0" };
                zeshia(val, path, true);
            } else {
                let val = if enabled == "Y" || enabled == "1" { "Y" } else { "N" };
                zeshia(val, path, true);
            }
        }
    }
}

fn apply_sched_features(features: &[&str]) {
    let path = "/sys/kernel/debug/sched_features";
    if Path::new(path).exists() {
        for feature in features {
            zeshia(feature, path, true);
        }
    }
}

fn parse_resolution() {
    let reso_prop = "persist.sys.azenithconf.resosettings";
    if getprop(reso_prop).is_empty() {
        if let Ok(out) = Command::new("wm").arg("size").output() {
            let s = String::from_utf8_lossy(&out.stdout);
            for word in s.split_whitespace() {
                if word.contains('x') {
                    let parts: Vec<&str> = word.split('x').collect();
                    if parts.len() == 2 && parts[0].chars().all(|c| c.is_digit(10)) && parts[1].chars().all(|c| c.is_digit(10)) {
                        setprop(reso_prop, word);
                        dlog(&format!("Detected resolution: {}", word));
                        dlog(&format!("Property {} set successfully", reso_prop));
                        break;
                    }
                }
            }
        } else {
            dlog("Failed to detect physical resolution");
        }
    }

    let schemeconfig = getprop("persist.sys.azenithconf.schemeconfig");
    if schemeconfig != "1000 1000 1000 1000" && !schemeconfig.is_empty() {
        let parts: Vec<&str> = schemeconfig.split_whitespace().collect();
        if parts.len() >= 4 {
            if let (Ok(r), Ok(g), Ok(b), Ok(s)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>(), parts[2].parse::<f64>(), parts[3].parse::<f64>()) {
                let rf = r / 1000.0;
                let gf = g / 1000.0;
                let bf = b / 1000.0;
                let sf = s / 1000.0;

                let _ = Command::new("service").args(&[
                    "call", "SurfaceFlinger", "1015", "i32", "1",
                    "f", &rf.to_string(), "f", "0", "f", "0", "f", "0",
                    "f", &gf.to_string(), "f", "0", "f", "0", "f", "0",
                    "f", &bf.to_string(), "f", "0", "f", "0", "f", "0",
                    "f", "1"
                ]).output();
                let _ = Command::new("service").args(&["call", "SurfaceFlinger", "1022", "f", &sf.to_string()]).output();
            }
        }
    }
}
