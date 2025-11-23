use defmt::{error, info, warn, Format};
use embassy_time::{Duration, Timer};
use esp_radio::wifi::{WifiController, WifiEvent};

use crate::events::{Event, send_event};

#[derive(Debug, Clone, Copy, Format)]
pub enum WifiState {
    Connecting,
    Connected,
    Disconnected,
}

impl From<WifiState> for &str {
    fn from(state: WifiState) -> Self {
        match state {
            WifiState::Connecting => "Wifi: connecting",
            WifiState::Connected => "Wifi: connected",
            WifiState::Disconnected => "Wifi: disconnected",
        }
    }
}

#[embassy_executor::task]
pub async fn wifi_task(mut controller: WifiController<'static>) {
    info!("wifi_task: starting driver");

    if let Err(e) = controller.start_async().await {
        error!("wifi_task: start_async failed: {:?}", e);
        send_event(Event::WifiStatus(WifiState::Disconnected)).await;
        return;
    }

    info!("wifi_task: driver started, connecting as STA");

    send_event(Event::WifiStatus(WifiState::Connecting)).await;

    loop {
        info!("wifi_task: connecting to AP…");

        match controller.connect_async().await {
            Ok(()) => {
                info!("wifi: connected, waiting for disconnect");
                send_event(Event::WifiStatus(WifiState::Connected)).await;

                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                warn!("wifi: STA disconnected, retrying in 5s");
                send_event(Event::WifiStatus(WifiState::Disconnected)).await;

                Timer::after(Duration::from_secs(5)).await;
            }
            Err(e) => {
                warn!("wifi: connect_async failed: {:?}", e);
                send_event(Event::WifiStatus(WifiState::Disconnected)).await;
                Timer::after(Duration::from_secs(5)).await;
            }
        }
    }
}
