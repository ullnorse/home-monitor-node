use embassy_time::{Duration, Timer};

use crate::events::{Event, send_event};
use crate::tasks::SensorHandle;

#[embassy_executor::task]
pub async fn sensor_task(mut sensor: SensorHandle) {
    loop {
        if let Ok(reading) = sensor.read() {
            send_event(Event::SensorReading(reading)).await;
        }

        Timer::after(Duration::from_secs(1)).await;
    }
}
