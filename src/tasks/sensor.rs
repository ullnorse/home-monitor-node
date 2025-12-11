use defmt::warn;
use embassy_time::Timer;

use crate::events::{Event, send_event};
use crate::tasks::SensorHandle;

const SENSOR_POLLING_RATE_MS: u64 = 1000;

#[embassy_executor::task]
pub async fn sensor_task(mut sensor: SensorHandle) {
    loop {
        match sensor.read() {
            Ok(reading) => {
                send_event(Event::SensorReading(reading)).await;
            }
            Err(e) => {
                warn!("Sensor read error: {}", e);
            }
        }

        //send_event(Event::SensorReading(crate::drivers::sht3x::Sht3xReading {temperature: 69f64, humidity: 68f64})).await;

        Timer::after_millis(SENSOR_POLLING_RATE_MS).await;
    }
}
