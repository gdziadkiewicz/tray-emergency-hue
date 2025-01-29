use std::{error::Error, net::{IpAddr}};

use hues::{api::HueAPIError, service::{Bridge, ResourceIdentifier, ResourceType}};
use serde::Deserialize;
use tokio::{sync::mpsc, task::JoinHandle};
use tray_item::{IconSource, TIError, TrayItem};

use config::Config;

#[derive(Debug)]
struct MyHueAPIError(HueAPIError);

impl From<HueAPIError> for MyHueAPIError {
    fn from(err: HueAPIError) -> MyHueAPIError {
        MyHueAPIError(err)
    }
}

impl std::fmt::Display for MyHueAPIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HueAPIError: {:?}", self.0)
    }
}

impl Error for MyHueAPIError {}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct AppConfig {
    bridge_address: String,
    app_key: String,
    on_button_rid: String,
    off_button_rid: String,
    light_id: String,
}

fn get_config() -> Result<AppConfig, config::ConfigError> {
    let config = Config::builder()
        .add_source(
            config::Environment::with_prefix("HUE")
                .try_parsing(true)
                ,
        )
        .build()?;

    config.try_deserialize::<AppConfig>()
}

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
            LevelFilter::Debug,
            Config::default(),
            File::create("my_rust_binary.log")?,
        ),
    ])?;
    Ok(())
}

fn setup_tray_app(tx: &mpsc::Sender<Message>) -> Result<TrayItem, TIError> {
    let mut tray: TrayItem = TrayItem::new(
        "Tray Example",
        IconSource::Resource("tray-default"),
    )?;

    tray.add_label("Tray Label")?;
    tray.inner_mut().set_tooltip("Piesek")?;

    let quit_tx = tx.clone();
    tray.add_menu_item("Quit", move || {
        quit_tx.blocking_send(Message::Quit).unwrap();
    })?;
    Ok(tray)
}

fn start_event_loop(mut rx: mpsc::Receiver<Message>, mut tray_item: TrayItem, mut bridge: Bridge, light_id: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Some(Message::Quit) => {
                    log::info!("Quit");
                    break;
                },
                None => {
                    log::error!("Channel closed");
                    break;
                },
                Some(Message::StartAlert) => {
                    log::info!("StartAlert");
                    show_notification();
                    start_light(&bridge, &light_id).await.unwrap();
                    tray_item.set_icon(IconSource::Resource("red")).unwrap()
                },
                Some(Message::StopAlert) => {
                    log::info!("StopAlert");
                    //show_notification();
                    stop_light(&bridge, &light_id).await.unwrap();
                    tray_item.set_icon(IconSource::Resource("green")).unwrap()

                }
            }
        }
    })
}

async fn start_light(bridge: &Bridge, light_id: &str, ) -> Result<(), HueAPIError> {
    bridge.light(light_id).unwrap().on().await?;
    Ok(())
}

async fn stop_light(bridge: &Bridge, light_id: &str, ) -> Result<(), HueAPIError> {
    bridge.light(light_id).unwrap().off().await?;
    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger()?;
    let AppConfig{ bridge_address, app_key, on_button_rid, off_button_rid, light_id } = get_config()?;
    let bridge_address = bridge_address.parse()?;
    let (tx, mut rx) = mpsc::channel(1);
    let mut tray_item = setup_tray_app(&tx)?;
    let bridge = setup_bridge(&tx, bridge_address, &app_key, &on_button_rid, &off_button_rid).await?;
    let event_loop = start_event_loop(rx, tray_item, bridge, light_id);
    event_loop.await?;
    Ok(())
}

async fn setup_bridge(tx: &mpsc::Sender<Message>, bridge_address: IpAddr, app_key: &str, on_button_rid: &str, off_button_rid: &str) -> Result<Bridge, MyHueAPIError> {
    let on_button = ResourceIdentifier { rid: on_button_rid.to_string(), rtype: ResourceType::Button };
    let off_button = ResourceIdentifier { rid: off_button_rid.to_string(), rtype: ResourceType::Button };
    let bridge = Bridge::new(bridge_address, app_key);
    bridge.refresh().await?;
    let tx = tx.clone();
    Ok(bridge.listen(move |event|{
        if event.contains(&on_button) {
            tx.try_send(Message::StartAlert).unwrap();
        } else if event.contains(&off_button) {
            tx.try_send(Message::StopAlert).unwrap();
        }
    }).await)
}

fn show_notification() {
    use notify_rust::Notification;
    Notification::new()
        .appname("TestAppName")
        .summary("ALARM!")
        .body("Test!!!")
        .show()
        .unwrap();
}
