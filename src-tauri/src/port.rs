use std::collections::HashMap;
use std::error::Error;
use std::os::windows::process::ExitStatusExt;
use std::time::Duration;
use std::{path::PathBuf, process::Output};

use btleplug::api::ValueNotification;
use serialport::{SerialPort, SerialPortInfo};
use regex::Regex;
use tokio::process::Command;
use log::{error, debug};

pub struct PortManager {
    resource_path: Option<PathBuf>,
    other_port: Option<String>,
}

impl PortManager {
    pub async fn new() -> Self {
        let manager = Self {
            resource_path: None,

            other_port: None,
        };
        manager
    }

    pub async fn init(&self) -> (Box<dyn SerialPort>, std::string::String) {
        let port_info = match Self::check_ports().await {
            Ok(ports) => ports,
            Err(err) => {
                error!("Error getting available ports: {}", err);
                None
            }
        };

        if port_info.is_none() {
            self.preinstall().await;
            self.remove_ports().await;
            self.install_ports().await;
        }

        let mut port = serialport::new("CNCA0", 115200)
            .timeout(Duration::from_millis(10))
            .open()
            .expect("Failed to open port");

        let other_port = self.get_ports("CNCB0").await;

        debug!("Other port: {}", other_port);
        (port, other_port)
    }

    async fn preinstall(&self) {
        let output = self.run_command(&["preinstall"]).await;

        if !output.status.success() {
            let output_str =
                String::from_utf8(output.stdout).expect("Failed to parse output as UTF-8");
            error!("Error preinstalling: {}", output_str);
        }
    }

    async fn run_command(&self, args: &[&str]) -> Output {
        debug!("Running command: {:?}", args);
        if let Some(resource_path) = &self.resource_path {
            let mut command = Command::new(resource_path.join("setupc.exe"));
            command.current_dir(resource_path);

            let output = command
                .args(args)
                .output()
                .await
                .expect("failed to execute process");

            debug!("output: {}", String::from_utf8_lossy(&output.stdout));
            output
        } else {
            error!("resource_path is None");
            Output {
                status: std::process::ExitStatus::from_raw(1),
                stdout: Vec::new(),
                stderr: Vec::new(),
            }
        }
    }

    pub async fn get_ports(&self, port: &str) -> String {
        let output = self.run_command(&["list"]).await;

        let output_str = String::from_utf8(output.stdout).expect("Failed to parse output as UTF-8");

        let port_name_re =
            Regex::new(r"(CNCA\d+|CNCB\d+)(?:.*PortName=([^\s,]+))?(?:.*RealPortName=(COM\d+))?")
                .expect("Failed to compile regex");
        let mut port_names: HashMap<String, String> = HashMap::new();

        for line in output_str.lines() {
            if let Some(port_name_caps) = port_name_re.captures(line) {
                debug!("Found {:?}", port_name_caps);
                let port_key = port_name_caps.get(1).unwrap().as_str();

                let real_port_name = if let Some(real_port_name_match) = port_name_caps.get(3) {
                    real_port_name_match.as_str()
                } else if let Some(port_name_match) = port_name_caps.get(2) {
                    port_name_match.as_str()
                } else {
                    continue;
                };

                port_names.insert(port_key.to_owned(), real_port_name.to_owned());
            }
        }
        if let Some(port_name) = port_names.get(port) {
            port_name.to_owned()
        } else {
            String::new()
        }
    }

    pub async fn install_ports(&self) {
        self.run_command(&["install", "0", "PortName=CNCA0", "PortName=COM#"])
            .await;
        self.after_install().await;
    }

    pub async fn remove_ports(&self) -> Output {
        self.run_command(&["remove", "0"]).await
    }

    async fn after_install(&self) {
        self.run_command(&["change", "CNCB0", "EmuOverrun=yes"])
            .await;
        self.run_command(&["change", "CNCA0", "EmuBR=yes"]).await;
    }

    pub async fn set_resource_path(&mut self, resource_path: PathBuf) {
        self.resource_path = Some(resource_path);
    }

    async fn check_ports() -> Result<Option<SerialPortInfo>, Box<dyn Error>> {
        let ports = serialport::available_ports()?;
        let port_info = ports.into_iter().find(|p| p.port_name.contains("CNCA0"));
        Ok(port_info)
    }
}
