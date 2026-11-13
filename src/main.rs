use evdev::EventType;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const KEY_HOME: u16 = 102;
const KEY_X: u16 = 45;

fn main() {
    let armed = AtomicBool::new(false);
    let (tx, rx) = mpsc::channel::<(u16, i32)>();

    let mut devices = Vec::new();
    for (_, dev) in evdev::enumerate() {
        if dev.supported_events().contains(EventType::KEY) {
            devices.push(dev);
        }
    }

    for mut dev in devices {
        let tx = tx.clone();
        thread::spawn(move || loop {
            if let Ok(events) = dev.fetch_events() {
                for ev in events {
                    if ev.event_type() == EventType::KEY {
                        let _ = tx.send((ev.code(), ev.value()));
                    }
                }
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        });
    }

    let mut net_on = true;
    println!("net-toggle: Home arms/disarms. X toggles network instantly.");
    for (code, value) in rx {
        if value != 1 {
            continue;
        }
        if code == KEY_HOME {
            let new = !armed.load(Ordering::SeqCst);
            armed.store(new, Ordering::SeqCst);
            let msg = format!("Hotkey {}", if new { "ARMED" } else { "DISARMED" });
            let _ = std::process::Command::new("notify-send").args(["-t", "1500", "net-toggle", &msg]).status();
            println!("{}", msg);
        } else if code == KEY_X && armed.load(Ordering::SeqCst) {
            net_on = !net_on;
            let arg = if net_on { "on" } else { "off" };
            let status = std::process::Command::new("nmcli")
                .args(["--wait", "0", "networking", arg])
                .status();
            let msg = format!("Network {}", if net_on { "ON" } else { "OFF" });
            let _ = std::process::Command::new("notify-send")
                .args(["-t", "2000", "net-toggle", &msg])
                .status();
            println!("{} ({})", msg, if status.map(|s| s.success()).unwrap_or(false) { "ok" } else { "FAIL" });
        }
    }
}
