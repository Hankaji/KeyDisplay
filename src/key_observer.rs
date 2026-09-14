use evdev::{Device, EventSummary, EventType, KeyCode};
use std::io::Write;
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};
use std::{io, thread};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Down,
    Up,
    Repeat,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub key: KeyCode,
    pub state: KeyState,
}

pub struct KeyObserver {
    rx: Receiver<KeyEvent>,
}

impl KeyObserver {
    pub fn spawn() -> Option<Self> {
        // Permission access
        if !Self::has_access() {
            if !Self::prompt_and_fix_permissions() {
                return None;
            }
            eprintln!(
                "You've been added to the 'input' group. Log out and back in \
                 (or run `newgrp input` in a fresh shell), then run this again."
            );
            return None;
        }

        let (tx, rx) = mpsc::channel();
        for device in Self::find_keyboards() {
            Self::spawn_reader(device, tx.clone());
        }
        Some(Self { rx })
    }

    fn has_access() -> bool {
        let Ok(entries) = std::fs::read_dir("/dev/input") else {
            return false;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let is_event = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("event"));
            if is_event && Device::open(&path).is_ok() {
                return true;
            }
        }
        false
    }

    fn prompt_and_fix_permissions() -> bool {
        print!(
            "No permission to read keyboard input devices.\n\
             Add your user to the 'input' group now? (requires sudo) [y/N] "
        );
        io::stdout().flush().ok();

        let mut answer = String::new();
        if io::stdin().read_line(&mut answer).is_err() {
            return false;
        }
        if !answer.trim().eq_ignore_ascii_case("y") {
            eprintln!("Skipped. Can't read keyboard devices without access.");
            return false;
        }

        let username = std::env::var("USER")
            .or_else(|_| std::env::var("LOGNAME"))
            .unwrap_or_default();
        if username.is_empty() {
            eprintln!("Couldn't determine current username.");
            return false;
        }

        let status = Command::new("sudo")
            .args(["usermod", "-aG", "input", &username])
            .status();

        match status {
            Ok(s) if s.success() => true,
            Ok(_) => {
                eprintln!("usermod failed.");
                false
            }
            Err(e) => {
                eprintln!("Failed to run sudo: {e}");
                false
            }
        }
    }

    fn find_keyboards() -> Vec<Device> {
        evdev::enumerate()
            .filter(|(_, dev)| {
                dev.supported_events().contains(EventType::KEY)
                    && dev
                        .supported_keys()
                        .is_some_and(|keys| keys.contains(KeyCode::KEY_A))
            })
            .map(|(_, dev)| dev)
            .collect()
    }

    fn spawn_reader(mut device: Device, tx: Sender<KeyEvent>) {
        thread::spawn(move || {
            loop {
                let events = match device.fetch_events() {
                    Ok(events) => events,
                    Err(e) => {
                        eprintln!("evdev read error: {e}");
                        return;
                    }
                };
                for ev in events {
                    if let EventSummary::Key(_, key, value) = ev.destructure() {
                        let state = match value {
                            0 => KeyState::Up,
                            1 => KeyState::Down,
                            2 => KeyState::Repeat,
                            _ => continue,
                        };
                        if tx.send(KeyEvent { key, state }).is_err() {
                            return; // receiver gone, stop this thread
                        }
                    }
                }
            }
        });
    }

    /// Blocks until the next key event.
    pub fn recv(&self) -> Option<KeyEvent> {
        self.rx.recv().ok()
    }
}

pub fn key_code_to_name(key: evdev::KeyCode) -> String {
    use evdev::KeyCode;
    match key {
        KeyCode::KEY_LEFTSHIFT | KeyCode::KEY_RIGHTSHIFT => "shift".into(),
        KeyCode::KEY_LEFTCTRL | KeyCode::KEY_RIGHTCTRL => "ctrl".into(),
        KeyCode::KEY_LEFTALT | KeyCode::KEY_RIGHTALT => "alt".into(),
        KeyCode::KEY_LEFTMETA | KeyCode::KEY_RIGHTMETA => "super".into(),
        KeyCode::KEY_SPACE => "space".into(),
        other => format!("{:?}", other)
            .trim_start_matches("KEY_")
            .to_lowercase(),
    }
}
