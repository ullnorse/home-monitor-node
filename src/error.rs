use defmt::Format;
use thiserror::Error;

#[derive(Error, Format, Debug)]
pub enum AppError {
    #[error("failed to spawn embassy task")]
    TaskSpawnFailed(#[from] embassy_executor::SpawnError),

    #[error("sht3x failed to initialize")]
    Sht3x,

    #[error("failed to initialize i2c")]
    I2cConfigError(#[from] esp_hal::i2c::master::ConfigError),

    #[error("i2c was already taken")]
    I2cAlreadyTaken,

    #[error("failed to initialize display")]
    DisplayInit,

    #[error("failed to draw to display")]
    DisplayDraw,
}

pub type Result<T> = core::result::Result<T, AppError>;
