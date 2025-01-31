use std::{error::Error, net::IpAddr, time::Duration};

use hues::{
    api::HueAPIError,
    prelude::LightCommand,
    service::{Bridge, ResourceIdentifier, ResourceType},
};
use tokio::{
    sync::mpsc::{self, error::TryRecvError},
    task::JoinHandle,
    time::sleep,
};

#[derive(Debug)]
pub struct MyHueAPIError(pub HueAPIError);

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

pub enum Button {
    On,
    Off,
}

pub async fn setup_bridge(
    bridge_address: IpAddr,
    app_key: &str,
    on_button_rid: &str,
    off_button_rid: &str,
    cb: impl Fn(Button) + Send + Sync + 'static,
) -> Result<Bridge, MyHueAPIError> {
    let on_button = ResourceIdentifier {
        rid: on_button_rid.to_string(),
        rtype: ResourceType::Button,
    };
    let off_button = ResourceIdentifier {
        rid: off_button_rid.to_string(),
        rtype: ResourceType::Button,
    };
    let bridge = Bridge::new(bridge_address, app_key);
    bridge.refresh().await?;
    Ok(bridge
        .listen(move |event| {
            if event.contains(&on_button) {
                cb(Button::On);
            } else if event.contains(&off_button) {
                cb(Button::Off);
            }
        })
        .await)
}

pub fn start_blinking_handler(
    bridge: Bridge,
    light_id: String,
) -> (JoinHandle<()>, mpsc::Sender<bool>) {
    let (light_loop_tx, mut light_loop_rx) = mpsc::channel(1);
    let blinking_handle = tokio::spawn(async move {
        loop {
            match light_loop_rx.recv().await {
                Some(true) => loop {
                    bridge
                        .light(&light_id)
                        .unwrap()
                        .send(&[
                            LightCommand::color_from_rgb([255, 0, 0]),
                            LightCommand::On(true),
                        ])
                        .await
                        .unwrap();
                    sleep(Duration::from_secs(1)).await;
                    bridge.light(&light_id).unwrap().off().await.unwrap();
                    sleep(Duration::from_secs(1)).await;
                    match light_loop_rx.try_recv() {
                        Ok(false) => break,
                        Ok(true) | Err(TryRecvError::Empty) => {}
                        Err(TryRecvError::Disconnected) => {
                            log::error!("Channel closed");
                            break;
                        }
                    }
                },
                Some(false) => {}
                None => {
                    log::error!("Channel closed");
                    break;
                }
            }
        }
    });
    (blinking_handle, light_loop_tx)
}
