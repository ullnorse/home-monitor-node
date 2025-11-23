use defmt::info;

use crate::events::{Event, receive_event};
use crate::tasks::display::{DisplayData, update_display_text};
use crate::tasks::http_client::send_sensor_data;
use crate::tasks::wifi::WifiState;

#[embassy_executor::task]
pub async fn orchestrate_task() {
    let mut wifi_state = WifiState::Connecting;

    loop {
        let event = receive_event().await;

        match event {
            Event::SensorReading(data) => {
                info!("Received sensor data");
                update_display_text(DisplayData::new(data.temperature, data.humidity, wifi_state));
                send_sensor_data(data);
            }

            Event::WifiStatus(state) => {
                 info!("WiFi state changed: {}", state);
                 wifi_state = state;
            }
        }
    }
}
