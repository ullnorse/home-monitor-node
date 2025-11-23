use esp_radio::wifi::WifiDevice;

#[embassy_executor::task]
pub async fn net_task(mut runner: embassy_net::Runner<'static, WifiDevice<'static>>) {
    runner.run().await;
}
