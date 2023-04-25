use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager, PeripheralId};
use futures::stream::StreamExt;
use log::{debug, error};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

const SERVICE_UUID: Uuid = Uuid::from_u128(0x0000ffa0_0000_1000_8000_00805f9b34fb);
const CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x0000ffa1_0000_1000_8000_00805f9b34fb);

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Device {
    pub id: String,
    pub name: String,
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({:?})", self.name, self.id)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum BleEvent {
    DeviceDiscovered(Vec<Device>),
    DeviceConnected(Device),
    DeviceDisconnected,
    DeviceUpdated(Device),
    DeviceNotification(Vec<u8>),
    DeviceError(String),
}

pub struct BleManager {
    pub central: Adapter,

    devices: Arc<Mutex<HashMap<String, (PeripheralId, Device)>>>,
}

impl BleManager {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let central = Self::get_central().await?;
        let ble_manager = BleManager {
            central,
            devices: Default::default(),
        };
        Ok(ble_manager)
    }

    pub async fn init_event_loop(
        &self,
        device_tx: mpsc::Sender<BleEvent>,
    ) -> Result<(), Box<dyn Error>> {
        let mut events = self.central.events().await?;

        let central_copy = self.central.clone();

        let devices_clone = self.devices.clone();

        tokio::spawn(async move {
            while let Some(event) = events.next().await {
                match event {
                    CentralEvent::DeviceDiscovered(id) => {
                        let peripheral = central_copy.peripheral(&id).await.unwrap();
                        debug!("Discovered: {:?}", peripheral);
                        let property = peripheral.properties().await.unwrap().unwrap();
                        let name = property.local_name;
                        let id = peripheral.address().to_string();

                        if let Some(name) = name {
                            if !name.contains("BioSignal") {
                                continue;
                            }
                            debug!("Discovered: {:?}", peripheral);
                            let device = Device {
                                id: id.clone(),
                                name,
                            };
                            devices_clone
                                .lock()
                                .await
                                .insert(id.clone(), (peripheral.id(), device));

                            let devices = devices_clone.lock().await;
                            let devices = devices
                                .clone()
                                .into_iter()
                                .map(|(_, device)| Device {
                                    id: device.1.id.clone(),
                                    name: device.1.name.clone(),
                                })
                                .collect::<Vec<Device>>();

                            if let Err(err) =
                                device_tx.send(BleEvent::DeviceDiscovered(devices)).await
                            {
                                error!("Error sending device discovered: {:?}", err);
                            }
                        }
                    }
                    // CentralEvent::DeviceUpdated(id) => todo!(),
                    CentralEvent::DeviceConnected(id) => {
                        let peripheral = central_copy.peripheral(&id).await.unwrap();
                        debug!("Connected: {:?}", peripheral);
                        let property = peripheral.properties().await.unwrap().unwrap();
                        let id = peripheral.address().to_string();
                        let device = devices_clone
                            .lock()
                            .await
                            .get(&id)
                            .expect("No device found")
                            .1
                            .clone();

                        peripheral
                            .discover_services()
                            .await
                            .expect("Failed to discover services");
                        let characteristics = peripheral.characteristics();

                        let rx_characteristic = match characteristics.iter().find(|c| {
                            c.service_uuid == SERVICE_UUID && c.uuid == CHARACTERISTIC_UUID
                        }) {
                            Some(c) => c,
                            None => {
                                error!("No characteristic found");
                                peripheral.disconnect().await.unwrap();
                                device_tx
                                    .send(BleEvent::DeviceError(
                                        "No characteristic found".to_string(),
                                    ))
                                    .await
                                    .unwrap();
                                continue;
                            }
                        };

                        peripheral
                            .subscribe(rx_characteristic)
                            .await
                            .expect("Failed to subscribe");

                        let mut notifications = peripheral.notifications().await.unwrap();
                        let device_tx_clone = device_tx.clone();

                        tokio::spawn(async move {
                            while let Some(notification) = notifications.next().await {
                                if let Err(err) = device_tx_clone
                                    .send(BleEvent::DeviceNotification(notification.value))
                                    .await
                                {
                                    error!("Error sending notification: {:?}", err);
                                    break;
                                }
                            }
                        });

                        if let Err(err) = device_tx.send(BleEvent::DeviceConnected(device)).await {
                            error!("Error sending device connected event: {:?}", err);
                        }
                    }
                    CentralEvent::DeviceDisconnected(id) => {
                        if let Err(err) = device_tx.send(BleEvent::DeviceDisconnected).await {
                            error!("Error sending device disconnect event: {:?}", err);
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    pub async fn start_scan(&mut self) -> Result<(), Box<dyn Error>> {
        self.stop_scan().await?;
        self.central.start_scan(Default::default()).await?;
        Ok(())
    }

    async fn get_central() -> Result<Adapter, Box<dyn Error>> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;

        adapters
            .into_iter()
            .next()
            .ok_or_else(|| "No Bluetooth adapters found".into())
    }

    pub async fn connect_device(&self, id: String) -> Result<(), Box<dyn Error>> {
        let peripheral_id = self.devices.lock().await;
        let peripheral_id = peripheral_id.get(&id).ok_or("Device not found")?;

        let peripheral = self.central.peripheral(&peripheral_id.0).await?;
        peripheral.connect().await?;

        Ok(())
    }

    pub async fn stop_scan(&self) -> Result<(), Box<dyn Error>> {
        self.central.stop_scan().await?;

        Ok(())
    }

    pub async fn disconnect_device(&self, id: String) -> Result<(), Box<dyn Error>> {
        let peripheral_id = self.devices.lock().await;
        let peripheral_id = peripheral_id.get(&id).ok_or("Device not found")?;

        let peripheral = self.central.peripheral(&peripheral_id.0).await?;
        peripheral.disconnect().await?;

        Ok(())
    }

    pub async fn get_devices(&self) -> Result<Vec<Device>, Box<dyn Error>> {
        let devices = self.devices.lock().await;
        let devices = devices
            .clone()
            .into_iter()
            .map(|(_, device)| Device {
                id: device.1.id.clone(),
                name: device.1.name.clone(),
            })
            .collect::<Vec<Device>>();

        Ok(devices)
    }
}
