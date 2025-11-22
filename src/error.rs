use defmt::Format;
use thiserror::Error;

#[derive(Error, Format, Debug)]
pub enum AppError {
    #[error("failed to spawn embassy task")]
    TaskSpawnFailed(#[from] embassy_executor::SpawnError),

    #[error("failed to initialize i2c")]
    I2cConfigError(#[from] esp_hal::i2c::master::ConfigError),

    #[error("i2c was already taken")]
    I2cAlreadyTaken,

    #[error("failed to initialize display")]
    Display,

    #[error("esp radio failed to initialize")]
    EspRadioInitFailed(#[from] esp_radio::InitializationError),

    #[error("esp wifi failed to initialize")]
    WifiInitFailed(#[from] esp_radio::wifi::WifiError),
}

pub type Result<T> = core::result::Result<T, AppError>;
