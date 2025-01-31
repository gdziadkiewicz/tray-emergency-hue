#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod hue;
mod notification;
mod tray;

use config::{get_config, AppConfig};
use hue::{setup_bridge, start_blinking_handler};
use notification::show_notification;
use tokio::{sync::mpsc, task::JoinHandle};
use tray::{change_icon, setup_tray_app, Icon};
use tray_item::TrayItem;

enum Message {
    Quit,
    StartAlert,
    StopAlert,
}

fn setup_logger() -> Result<(), Box<dyn std::error::Error>> {
    use log::LevelFilter;
    use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
    use std::fs::File;

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("my_rust_binary.log")?,
        ),
    ])?;
    Ok(())
}

fn start_event_loop(
    mut rx: mpsc::Receiver<Message>,
    mut tray_item: TrayItem,
    blinking_tx: mpsc::Sender<bool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Some(Message::Quit) => {
                    log::info!("Quit");
                    break;
                }
                None => {
                    log::error!("Channel closed");
                    break;
                }
                Some(Message::StartAlert) => {
                    log::info!("StartAlert");
                    show_notification("Alert", "Alert started");
                    blinking_tx.send(true).await.unwrap();
                    change_icon(&mut tray_item, &Icon::Red).unwrap()
                }
                Some(Message::StopAlert) => {
                    log::info!("StopAlert");
                    show_notification("Alert", "Alert stopped");
                    blinking_tx.send(false).await.unwrap();
                    change_icon(&mut tray_item, &Icon::Green).unwrap()
                }
            }
        }
    })
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger()?;

    let AppConfig {
        bridge_address,
        app_key,
        on_button_rid,
        off_button_rid,
        light_id,
    } = get_config()?;
    let bridge_address = bridge_address.parse()?;

    let (event_loop_tx, event_loop_rx) = mpsc::channel(1);
    let quit_tx = event_loop_tx.clone();
    let tray_item = setup_tray_app(move || quit_tx.blocking_send(Message::Quit).unwrap())?;
    let bridge = setup_bridge(
        bridge_address,
        &app_key,
        &on_button_rid,
        &off_button_rid,
        move |b| match b {
            hue::Button::On => {
                event_loop_tx.try_send(Message::StartAlert).unwrap();
            }
            hue::Button::Off => {
                event_loop_tx.try_send(Message::StopAlert).unwrap();
            }
        },
    )
    .await?;
    let (_blinking_handle, blinking_tx) = start_blinking_handler(bridge, light_id);
    let event_loop_handle = start_event_loop(event_loop_rx, tray_item, blinking_tx);
    event_loop_handle.await?;

    Ok(())
}
