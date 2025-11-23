use defmt::{info, warn};
use embassy_time::{Duration, Timer};

use crate::events::{Event, send_event};
use crate::tasks::SensorHandle;

const SENSOR_POLLING_RATE_MS: u64 = 1000;

#[embassy_executor::task]
pub async fn sensor_task(mut sensor: SensorHandle) {
    let mut cnt = 0;

    loop {
        match sensor.read() {
            Ok(reading) => {
                send_event(Event::SensorReading(reading)).await;
                info!("Sending sensor data {}", cnt);
                cnt += 1;
            }
            Err(e) => {
                warn!("Sensor read error: {}", e);
            }
        }

        Timer::after(Duration::from_millis(SENSOR_POLLING_RATE_MS)).await;
    }
}
