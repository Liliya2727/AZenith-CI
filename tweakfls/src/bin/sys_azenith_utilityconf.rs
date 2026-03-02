use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::path::Path;

fn getprop(prop_name: &str) -> String {
    if let Ok(output) = Command::new("getprop").arg(prop_name).output() {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    String::new()
}

fn setprop(prop_name: &str, value: &str) {
    let _ = Command::new("setprop")
        .arg(prop_name)
        .arg(value)
        .output();
}

fn debugmode() -> bool {
    getprop("persist.sys.azenith.debugmode") == "true"
}

fn az_log(message: &str) {
    if debugmode() {
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
        .arg("AZenith_Utility")
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

    // chmod 644
    if let Ok(mut perms) = fs::metadata(p).map(|m| m.permissions()) {
        perms.set_mode(0o644);
        let _ = fs::set_permissions(p, perms);
    }

    match OpenOptions::new().write(true).truncate(true).open(p) {
        Ok(mut file) => {
            if file.write_all(value.as_bytes()).is_ok() {
                az_log(&format!("Set /{} to {}", pathname, value));
            } else {
                az_log(&format!("Cannot write to /{} (permission denied)", pathname));
            }
        }
        Err(_) => {
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

fn setsgov(gov: &str) {
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
        dlog(&format!("Set current CPU Governor to {}", gov));
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
        dlog(&format!("Set current GPU Mali Governor to {}", gov));
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
    dlog(&format!("Set current IO Scheduler to {}", scheduler));
}


fn setthermalcore(state: &str) {
    if state == "1" {
        let _ = Command::new("sys.azenith-rianixiathermalcore")
            .spawn();
        std::thread::sleep(std::time::Duration::from_secs(1));

        let output = Command::new("pgrep")
            .arg("-f")
            .arg("sys.azenith-rianixiathermalcore")
            .output();
        if let Ok(out) = output {
            let pid = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !pid.is_empty() {
                dlog(&format!("Starting Thermalcore Service with pid {}", pid));
            } else {
                dlog("Thermalcore service started but PID not found");
            }
        } else {
            dlog("Thermalcore service started but PID not found");
        }
    } else {
        let _ = Command::new("pkill")
            .arg("-9")
            .arg("-f")
            .arg("sys.azenith-rianixiathermalcore")
            .output();
        dlog("Stopped Thermalcore service");
    }
}

fn fstrim_func() {
    let fstrim_state = getprop("persist.sys.azenithconf.fstrim");
    if fstrim_state == "1" {
        let mounts = ["/system", "/vendor", "/data", "/cache", "/metadata", "/odm", "/system_ext", "/product"];
        for mount in mounts.iter() {
            if let Ok(output) = Command::new("mountpoint").arg("-q").arg(mount).output() {
                if output.status.success() {
                    let _ = Command::new("fstrim").arg("-v").arg(mount).output();
                    az_log(&format!("Trimmed: {}", mount));
                } else {
                    az_log(&format!("Skipped (not mounted): {}", mount));
                }
            } else {
                az_log(&format!("Skipped (not mounted): {}", mount));
            }
        }
        dlog("Trimmed unused blocks");
    }
}

fn enable_dnd() {
    if let Ok(output) = Command::new("cmd").args(&["notification", "set_dnd", "priority"]).output() {
        if output.status.success() {
            dlog("DND enabled");
        } else {
            dlog("Failed to enable DND");
        }
    } else {
        dlog("Failed to enable DND");
    }
}

fn disable_dnd() {
    if let Ok(output) = Command::new("cmd").args(&["notification", "set_dnd", "off"]).output() {
        if output.status.success() {
            dlog("DND disabled");
        } else {
            dlog("Failed to disable DND");
        }
    } else {
        dlog("Failed to disable DND");
    }
}

fn setrefreshrates(rate: &str) {
    let _ = Command::new("settings").args(&["put", "system", "peak_refresh_rate", rate]).output();
    let min_rate = format!("{}.0", rate);
    let _ = Command::new("settings").args(&["put", "system", "min_refresh_rate", &min_rate]).output();
    dlog(&format!("Set current refresh rates to: {}hz", rate));
}

fn setrender(renderer: &str) {
    match renderer {
        "vulkan" => {
            setprop("debug.hwui.renderer", "skiavk");
        }
        "skiagl" => {
            setprop("debug.hwui.renderer", "skiagl");
        }
        _ => return,
    }
    dlog(&format!("Set current renderer to: {}", renderer));
}


fn read_current_ma() -> i32 {
    let files = [
        "/sys/class/power_supply/battery/current_now",
        "/sys/class/power_supply/battery/BatteryAverageCurrent",
        "/sys/class/power_supply/battery/input_current_now",
        "/sys/class/power_supply/usb/current_now",
    ];

    for file in files.iter() {
        if let Ok(content) = fs::read_to_string(file) {
            let mut val_str = content.trim().to_string();
            if val_str.starts_with('-') {
                val_str.remove(0);
            }
            if let Ok(val) = val_str.parse::<i32>() {
                if val > 1000 {
                    return val / 1000;
                } else {
                    return val;
                }
            }
        }
    }
    9999
}

fn ischarging() -> bool {
    if let Ok(content) = fs::read_to_string("/sys/class/power_supply/battery/status") {
        let status = content.trim();
        return status == "Charging" || status == "Full";
    }
    false
}

fn eval_env(key: &str) -> String {
    let mtk_bypass = "/sys/devices/platform/charger/bypass_charger";
    let mtk_current_cmd = "/proc/mtk_battery_cmd/current_cmd";
    let tran_aichg = "/sys/devices/platform/charger/tran_aichg_disable_charger";
    let mtk_disable_charger = "/sys/devices/platform/mt-battery/disable_charger";

    match key {
        "MTK_BYPASS_CHARGER" => mtk_bypass.to_string(),
        "MTK_BYPASS_CHARGER_ON" => "1".to_string(),
        "MTK_BYPASS_CHARGER_OFF" => "0".to_string(),
        "MTK_CURRENT_CMD" => mtk_current_cmd.to_string(),
        "MTK_CURRENT_CMD_ON" => "0 1".to_string(),
        "MTK_CURRENT_CMD_OFF" => "0 0".to_string(),
        "TRAN_AICHG" => tran_aichg.to_string(),
        "TRAN_AICHG_ON" => "1".to_string(),
        "TRAN_AICHG_OFF" => "0".to_string(),
        "MTK_DISABLE_CHARGER" => mtk_disable_charger.to_string(),
        "MTK_DISABLE_CHARGER_ON" => "1".to_string(),
        "MTK_DISABLE_CHARGER_OFF" => "0".to_string(),
        _ => "".to_string(),
    }
}

fn enable_bypass() -> bool {
    if !ischarging() {
        dlog("Skipping bypass charge: device not charging");
        return true;
    }

    let bypasspath = getprop("persist.sys.azenithconf.bypasspath");
    let key = bypasspath;
    let val_key = format!("{}_ON", key);
    let path = eval_env(&key);
    let onval = eval_env(&val_key);

    if path.is_empty() || onval.is_empty() {
        return false;
    }

    let max_try = 5;
    let mut current_try = 0;

    while current_try < max_try {
        current_try += 1;

        zeshia(&onval, &path, true);
        std::thread::sleep(std::time::Duration::from_secs(1));

        let cur_val = fs::read_to_string(&path).unwrap_or_default().trim().to_string();
        let cur_ma = read_current_ma();

        az_log(&format!("Bypass check [{}]: path={} current={}mA", current_try, cur_val, cur_ma));

        if cur_val == onval && cur_ma <= 10 {
            dlog(&format!("Bypass active, current {}mA", cur_ma));
            return true;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    let cur_ma = read_current_ma();
    dlog(&format!("Bypass failed after {} retries (current {}mA)", max_try, cur_ma));
    false
}

fn disable_bypass() {
    let bypasspath = getprop("persist.sys.azenithconf.bypasspath");
    let key = bypasspath;
    let val_key = format!("{}_OFF", key);
    let path = eval_env(&key);
    let offval = eval_env(&val_key);

    if !path.is_empty() && !offval.is_empty() {
        zeshia(&offval, &path, true);
    }
    dlog("Bypass charge disabled");
}

fn check_bypass() -> bool {
    if !ischarging() {
        dlog("Skipping bypass compatibility check: device not charging");
        return true;
    }

    let bypasspath = getprop("persist.sys.azenithconf.bypasspath");
    if !bypasspath.is_empty() && bypasspath != "UNSUPPORTED" {
        dlog(&format!("Bypass Charging path already set: {}", bypasspath));
        return true;
    }

    let bypasspathlist = [
        ("MTK_BYPASS_CHARGER", "/sys/devices/platform/charger/bypass_charger"),
        ("MTK_CURRENT_CMD", "/proc/mtk_battery_cmd/current_cmd"),
        ("TRAN_AICHG", "/sys/devices/platform/charger/tran_aichg_disable_charger"),
        ("MTK_DISABLE_CHARGER", "/sys/devices/platform/mt-battery/disable_charger")
    ];

    for (name, path) in bypasspathlist.iter() {
        if !Path::new(path).exists() {
            continue;
        }

        let on_val = eval_env(&format!("{}_ON", name));
        let off_val = eval_env(&format!("{}_OFF", name));

        dlog(&format!("Testing bypass charging path: {}", name));

        zeshia(&on_val, path, true);
        std::thread::sleep(std::time::Duration::from_secs(1));

        let cur_ma = read_current_ma();
        dlog(&format!("Charging current: {}mA", cur_ma));

        zeshia(&off_val, path, true);
        std::thread::sleep(std::time::Duration::from_secs(1));

        if cur_ma < 10 {
            dlog(&format!("Bypass Charging SUPPORTED via {}", name));
            setprop("persist.sys.azenithconf.bypasspath", name);
            return true;
        }
    }

    dlog("Bypass Charging unsupported: no effective path found");
    setprop("persist.sys.azenithconf.bypasspath", "UNSUPPORTED");
    false
}

fn save_log() {
    let _ = Command::new("sh").arg("-c").arg("
    log_file=\"/sdcard/AZenithLog_$(date +\"%Y-%m-%d_%H-%M\").txt\"
    echo \"$log_file\"
    module_ver=$(awk -F'=' '/version=/ {print $2}' /data/adb/modules/AZenith/module.prop 2>/dev/null)
    android_sdk=$(getprop ro.build.version.sdk)
    kernel_info=$(uname -r -m)
    fingerprint=$(getprop ro.build.fingerprint)
    device=$(getprop sys.azenith.device)
    chipset=$(getprop sys.azenith.soc)

    {
        echo \"##########################################\"
        echo \"             AZenith Process Log\"
        echo
        echo \"    Module: $module_ver\"
        echo \"    Android: $android_sdk\"
        echo \"    Kernel: $kernel_info\"
        echo \"    Fingerprint: $fingerprint\"
        echo \"    Device: $device\"
        echo \"    Chipset: $chipset\"
        echo \"##########################################\"
        echo
        cat /data/adb/.config/AZenith/debug/AZenith.log 2>/dev/null
    } >\"$log_file\"
    ").output();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let command = &args[1];
        match command.as_str() {
            "setsgov" => {
                if args.len() > 2 { setsgov(&args[2]); }
            }
            "setsGPUMali" => {
                if args.len() > 2 { sets_gpu_mali(&args[2]); }
            }
            "setsIO" => {
                if args.len() > 2 { sets_io(&args[2]); }
            }
            "setthermalcore" => {
                if args.len() > 2 { setthermalcore(&args[2]); }
            }
            "FSTrim" => fstrim_func(),
            "enableDND" => enable_dnd(),
            "disableDND" => disable_dnd(),
            "setrefreshrates" => {
                if args.len() > 2 { setrefreshrates(&args[2]); }
            }
            "setrender" => {
                if args.len() > 2 { setrender(&args[2]); }
            }
            "enableBypass" => { enable_bypass(); }
            "disableBypass" => disable_bypass(),
            "checkBypass" => { check_bypass(); }
            "saveLog" => save_log(),
            _ => {
                az_log(&format!("Unknown command {}", command));
            }
        }
    }
}
