use embassy_time::{Duration, Timer};

use crate::{
    app::TempSensor,
    core::environment_sensor::EnvironmentSensor,
    event::{Event, send_event},
};

#[embassy_executor::task]
pub async fn sensor_task(sensor: TempSensor) {
    sensor_loop(sensor).await;
}

async fn sensor_loop(mut sensor: impl EnvironmentSensor) {
    loop {
        if let Ok(reading) = sensor.read() {
            send_event(Event::SensorReading(reading)).await;
        }

        Timer::after(Duration::from_secs(1)).await;
    }
}
