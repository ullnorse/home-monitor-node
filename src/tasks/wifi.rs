use defmt::{error, info, warn};
use embassy_time::{Duration, Timer};
use esp_radio::wifi::{AuthMethod, ClientConfig, ModeConfig, WifiController, WifiEvent};

#[embassy_executor::task]
pub async fn wifi_task(mut controller: WifiController<'static>) {
    info!("wifi: starting driver");
    if let Err(e) = controller.start_async().await {
        error!("wifi: start_async failed: {:?}", e);
        // You can return or loop/retry here, up to you
        return;
    }
    info!("wifi: driver started, connecting as STA");

    loop {
        info!("wifi: connecting to AP…");
        match controller.connect_async().await {
            Ok(()) => {
                info!("wifi: connected, waiting for disconnect");
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                warn!("wifi: STA disconnected, retrying in 5s");
                Timer::after(Duration::from_secs(5)).await;
            }
            Err(e) => {
                warn!("wifi: connect_async failed: {:?}", e);
                Timer::after(Duration::from_secs(5)).await;
            }
        }
    }
}