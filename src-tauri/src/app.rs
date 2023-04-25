use crate::{
    ble::{BleEvent, BleManager, Device},
    port::PortManager,
};
use btleplug::api::ValueNotification;
use log::error;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::{mpsc, mpsc::Sender, Mutex};

pub struct AppState {
    pub ble_manager: BleManager,
    pub port_manager: PortManager,
    pub other_port: Option<String>,
}

pub type AppStateType = Arc<Mutex<AppState>>;

impl AppState {
    pub async fn new() -> AppStateType {
        let ble_manager = BleManager::new().await.unwrap();
        let port_manager = PortManager::new().await;

        Arc::new(Mutex::new(Self {
            ble_manager,
            port_manager,
            other_port: None,
        }))
    }

    pub async fn set_resource_path(&mut self, resource_path: PathBuf) {
        self.port_manager.set_resource_path(resource_path).await
    }

    pub async fn start_loop(&mut self, ui_tx: Sender<BleEvent>) {
        let (device_tx, mut device_rx) = mpsc::channel(100);
        self.ble_manager
            .init_event_loop(device_tx)
            .await
            .expect("Error init event loop");

        let (mut port, other_port) = self.port_manager.init().await;
        self.other_port = Some(other_port);

        tokio::spawn(async move {
            while let Some(event) = device_rx.recv().await {
                match event {
                    crate::ble::BleEvent::DeviceNotification(data) => {
                        if let Err(err) = port.write(&data) {
                            error!("Error sending Data to Port: {}", err);
                        }
                    }

                    _ => {
                        if let Err(err) = ui_tx.send(event).await {
                            error!("Error sending event to UI: {}", err);
                        }
                    }
                }
            }
        });
    }
}
