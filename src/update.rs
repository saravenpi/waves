use std::sync::mpsc::{Sender, Receiver, channel};

pub enum UpdateStatus {
    Checking,
    UpdateAvailable(String),
    NoUpdateAvailable,
    Downloading(u8),
    Installing,
    Success,
    Error(String),
}

pub struct UpdateChecker {
    pub status_receiver: Receiver<UpdateStatus>,
    status_sender: Sender<UpdateStatus>,
}

impl UpdateChecker {
    pub fn new() -> Self {
        let (status_sender, status_receiver) = channel();
        Self {
            status_receiver,
            status_sender,
        }
    }

    pub fn check_for_updates(&self) {
        let sender = self.status_sender.clone();
        std::thread::spawn(move || {
            let _ = sender.send(UpdateStatus::Checking);

            let current_version = env!("CARGO_PKG_VERSION");

            let result = std::panic::catch_unwind(|| {
                let repo_owner = "saravenpi";
                let repo_name = "Waves";

                let url = format!(
                    "https://api.github.com/repos/{}/{}/releases/latest",
                    repo_owner, repo_name
                );

                let response = ureq::get(&url)
                    .set("User-Agent", &format!("WAVES/{}", current_version))
                    .call();

                match response {
                    Ok(resp) => {
                        let json: serde_json::Value = resp.into_json().unwrap_or_default();
                        let latest_version = json["tag_name"]
                            .as_str()
                            .unwrap_or("")
                            .trim_start_matches('v');

                        if version_is_newer(current_version, latest_version) {
                            UpdateStatus::UpdateAvailable(latest_version.to_string())
                        } else {
                            UpdateStatus::NoUpdateAvailable
                        }
                    }
                    Err(e) => UpdateStatus::Error(format!("Failed to check for updates: {}", e)),
                }
            });

            match result {
                Ok(status) => {
                    let _ = sender.send(status);
                }
                Err(_) => {
                    let _ = sender.send(UpdateStatus::Error("Failed to check for updates".to_string()));
                }
            }
        });
    }

    pub fn perform_update(&self) {
        let sender = self.status_sender.clone();
        std::thread::spawn(move || {
            let _ = sender.send(UpdateStatus::Downloading(0));

            let result = std::panic::catch_unwind(|| {
                let current_version = env!("CARGO_PKG_VERSION");
                let repo_owner = "saravenpi";
                let repo_name = "Waves";

                let (target, bin_name) = if cfg!(target_os = "macos") {
                    ("macos", "waves")
                } else if cfg!(target_os = "linux") {
                    ("x86_64-unknown-linux-gnu", "waves")
                } else if cfg!(target_os = "windows") {
                    ("x86_64-pc-windows-msvc", "waves")
                } else {
                    return UpdateStatus::Error("Unsupported platform".to_string());
                };

                let updater = self_update::backends::github::Update::configure()
                    .repo_owner(repo_owner)
                    .repo_name(repo_name)
                    .bin_name(bin_name)
                    .target(target)
                    .current_version(current_version)
                    .build();

                match updater {
                    Ok(updater) => {
                        let _ = sender.send(UpdateStatus::Installing);

                        match updater.update() {
                            Ok(_) => UpdateStatus::Success,
                            Err(e) => UpdateStatus::Error(format!("Update failed: {}", e)),
                        }
                    }
                    Err(e) => UpdateStatus::Error(format!("Failed to configure updater: {}", e)),
                }
            });

            match result {
                Ok(status) => {
                    let _ = sender.send(status);
                }
                Err(_) => {
                    let _ = sender.send(UpdateStatus::Error("Update process crashed".to_string()));
                }
            }
        });
    }
}

fn version_is_newer(current: &str, latest: &str) -> bool {
    let current_parts: Vec<u32> = current
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let latest_parts: Vec<u32> = latest
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    for i in 0..3 {
        let current_part = current_parts.get(i).copied().unwrap_or(0);
        let latest_part = latest_parts.get(i).copied().unwrap_or(0);

        if latest_part > current_part {
            return true;
        } else if latest_part < current_part {
            return false;
        }
    }

    false
}
