use thiserror::Error;

#[derive(Clone, Copy, Debug)]
pub struct EnvironmentReading {
    pub temperature_c: f64,
    pub humidity_rh: f64,
}

#[derive(Error, Debug, Clone, Copy)]
pub enum EnvironmentSensorError {
    #[error("bus error")]
    Bus,
    #[error("timouet occured")]
    Timeout,
    #[error("invalid data")]
    InvalidData,
}

pub trait EnvironmentSensor {
    fn read(&mut self) -> Result<EnvironmentReading, EnvironmentSensorError>;
}
