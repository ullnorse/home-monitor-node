use defmt::info;
use esp_radio::wifi::WifiDevice;

#[embassy_executor::task]
pub async fn net_task(mut runner: embassy_net::Runner<'static, WifiDevice<'static>>) {
    runner.run().await;
}

#[embassy_executor::task]
pub async fn alive_task() {
    let mut cnt = 0;

    loop {
        info!("I am alive {}", cnt);
        cnt += 1;
        embassy_time::Timer::after_millis(5000).await;
    }
}
