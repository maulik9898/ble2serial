// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app::AppStateType;
use ble::Device;
use log::error;
use std::{error::Error, path::PathBuf, process::Output, sync::Arc};
use tauri::Manager;
use tauri::State;
use tauri_plugin_log::LogTarget;
use tokio::sync::Mutex;
use log::debug;

use crate::ble::BleEvent;
mod app;
mod ble;
mod port;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn start_scan(state: State<'_, AppStateType>) -> Result<(), String> {
    let mut state = state.lock().await;
    if let Err(err) = state.ble_manager.start_scan().await {
        println!("Error starting scan: {}", err);
        return Err(err.to_string());
    }
    Ok(())
}

#[tauri::command]
async fn stop_scan(state: State<'_, AppStateType>) -> Result<(), String> {
    state
        .lock()
        .await
        .ble_manager
        .stop_scan()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn connect(state: State<'_, AppStateType>, id: String) -> Result<(), String> {
    state
        .lock()
        .await
        .ble_manager
        .connect_device(id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn disconnect(state: State<'_, AppStateType>, id: String) -> Result<(), String> {
    state
        .lock()
        .await
        .ble_manager
        .disconnect_device(id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_other_port(state: State<'_, AppStateType>) -> Result<Option<String>, String> {
    let state = state.lock().await;
    let port = state.other_port.clone();
    Ok(port)
}

#[tauri::command]
async fn get_devices(state: State<'_, AppStateType>) -> Result<Vec<Device>, String> {
    let state = state.lock().await;
    let devices = state
        .ble_manager
        .get_devices()
        .await
        .map_err(|e| e.to_string())?;
    Ok(devices)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    // use that subscriber to process traces emitted after this point
    
    tauri::async_runtime::set(tokio::runtime::Handle::current());
    

    let (ui_tx, mut ui_rx) = tokio::sync::mpsc::channel::<BleEvent>(100);
    let app_state = app::AppState::new().await;
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview])
                .build(),
        )
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            start_scan,
            stop_scan,
            connect,
            disconnect,
            get_other_port,
            get_devices
        ])
        .setup(move |app| {
            let cloned = app_state.clone();
            let mut resource_path = app
                .path_resolver()
                .resource_dir()
                .expect("failed to resolve resource");

            resource_path.push("com0com");

            let window = app.get_window("main").unwrap();

            tauri::async_runtime::spawn(async move {
                {
                    let mut state = cloned.lock().await;
                    state.set_resource_path(resource_path).await;
                    state.start_loop(ui_tx.clone()).await;
                }

                debug!("Starting UI loop..");

                while let Some(event) = ui_rx.recv().await {
                    match event {
                        BleEvent::DeviceDiscovered(devices) => {
                            if let Err(err) = window.emit("devices", devices) {
                                error!("Error sending devices to UI: {}", err);
                            };
                        }
                        BleEvent::DeviceConnected(device) => {
                            if let Err(err) = window.emit("connected", device) {
                                error!("Error sending devices to UI: {}", err);
                            };
                        }
                        BleEvent::DeviceDisconnected => {
                            if let Err(err) = window.emit("disconnected", ()) {
                                error!("Error sending devices to UI: {}", err);
                            };
                        }
                        BleEvent::DeviceError(err) => {
                            if let Err(err) = window.emit("error", err) {
                                error!("Error sending devices to UI: {}", err);
                            };
                        }
                        _ => {}
                    }
                }
            });

            Ok(()) // This will print 'Guten Tag!' to the terminal
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
