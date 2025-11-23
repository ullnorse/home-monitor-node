use core::net::Ipv4Addr;

use defmt::{error, info, warn};
use embassy_net::tcp::{State as TcpState, TcpSocket};
use embassy_net::{IpEndpoint, Stack};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};
use static_cell::StaticCell;

use crate::drivers::sht3x::Sht3xReading;

// -----------------------------------------------------------------------------
// Signal from orchestrator
// -----------------------------------------------------------------------------

static HTTP_DATA_SIGNAL: Signal<CriticalSectionRawMutex, Sht3xReading> = Signal::new();

pub fn send_sensor_data(data: Sht3xReading) {
    HTTP_DATA_SIGNAL.signal(data);
}

async fn wait_for_reading() -> Sht3xReading {
    HTTP_DATA_SIGNAL.wait().await
}

// -----------------------------------------------------------------------------
// Config
// -----------------------------------------------------------------------------

const SERVER_IP: [u8; 4] = [192, 168, 100, 15];
const SERVER_PORT: u16 = 8080;

const CONNECT_TIMEOUT_SECS: u64 = 10;
const RETRY_DELAY_SECS: u64 = 5;
const MIN_INTERVAL_BETWEEN_SENDS_MS: u64 = 10000;

// Socket buffers must be 'static
static RX_BUF: StaticCell<[u8; 1024]> = StaticCell::new();
static TX_BUF: StaticCell<[u8; 512]> = StaticCell::new();

fn server_endpoint() -> IpEndpoint {
    let ip = Ipv4Addr::new(SERVER_IP[0], SERVER_IP[1], SERVER_IP[2], SERVER_IP[3]);
    IpEndpoint::new(ip.into(), SERVER_PORT)
}

fn build_json_body<'a>(buf: &'a mut [u8; 128], reading: &Sht3xReading) -> &'a str {
    format_no_std::show(
        buf,
        format_args!(
            r#"{{"temperature":{:.2},"humidity":{:.2}}}"#,
            reading.temperature,
            reading.humidity
        ),
    )
    .unwrap()
}

fn build_content_length<'a>(buf: &'a mut [u8; 8], len: usize) -> &'a str {
    format_no_std::show(buf, format_args!("{}", len)).unwrap()
}

fn build_http_request<'a>(buf: &'a mut [u8; 256], content_length: &str) -> &'a str {
    // NOTE: header lines must start at column 0 (no leading spaces)
    format_no_std::show(
        &mut buf[..],
        format_args!(
            "POST /readings HTTP/1.1\r\n\
Host: local\r\n\
Content-Type: application/json\r\n\
Content-Length: {}\r\n\
Connection: close\r\n\
\r\n",
            content_length
        ),
    )
    .unwrap()
}

// -----------------------------------------------------------------------------
// Task
// -----------------------------------------------------------------------------

#[embassy_executor::task]
pub async fn http_client_task(stack: &'static Stack<'static>) {
    info!("http_client: task start");
    stack.wait_config_up().await;
    if let Some(cfg) = stack.config_v4() {
        info!("http_client: network up, my IP = {}", cfg.address);
    }

    let rx_buf = RX_BUF.init([0u8; 1024]);
    let tx_buf = TX_BUF.init([0u8; 512]);

    let mut socket = TcpSocket::new(*stack, rx_buf, tx_buf);
    socket.set_timeout(Some(Duration::from_secs(CONNECT_TIMEOUT_SECS)));

    loop {
        info!("http_client: waiting for reading");
        let reading = wait_for_reading().await;
        info!("http_client: got sensor reading, preparing to send");

        // Reconnect if needed
        if matches!(socket.state(), TcpState::Closed) {
            let remote = server_endpoint();
            info!(
                "http_client: connecting to {}:{}",
                remote.addr, remote.port
            );

            if let Err(e) = socket.connect(remote).await {
                warn!("http_client: connect error: {:?}", e);
                Timer::after(Duration::from_secs(RETRY_DELAY_SECS)).await;
                continue;
            }

            info!("http_client: connected");
        }

        let start = Instant::now();

        // Build request
        let mut json_buf = [0u8; 128];
        let mut len_buf = [0u8; 8];
        let mut req_buf = [0u8; 256];

        let json = build_json_body(&mut json_buf, &reading);
        let content_length = build_content_length(&mut len_buf, json.len());
        let request = build_http_request(&mut req_buf, content_length);

        if let Err(e) = socket.write(request.as_bytes()).await {
            error!("http_client: write headers error: {:?}", e);
            socket.close();
            Timer::after(Duration::from_secs(RETRY_DELAY_SECS)).await;
            continue;
        }

        if let Err(e) = socket.write(json.as_bytes()).await {
            error!("http_client: write body error: {:?}", e);
            socket.close();
            Timer::after(Duration::from_secs(RETRY_DELAY_SECS)).await;
            continue;
        }

        if let Err(e) = socket.flush().await {
            error!("http_client: flush error: {:?}", e);
            socket.close();
        } else {
            info!("http_client: POST sent");
            // Close after each request; next loop will reconnect
            socket.close();
        }

        // Enforce a minimum interval between sends
        let elapsed = Instant::now() - start;
        if elapsed < Duration::from_millis(MIN_INTERVAL_BETWEEN_SENDS_MS) {
            let remaining = Duration::from_millis(MIN_INTERVAL_BETWEEN_SENDS_MS) - elapsed;
            Timer::after(remaining).await;
        }
    }
}
