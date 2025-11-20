use defmt::Format;
use thiserror::Error;

use crate::core::environment_sensor::EnvironmentReading;

#[derive(Error, Debug, Format)]
pub enum DisplayError {
    #[error("initialization failed")]
    Init,
    #[error("draw failed")]
    Draw,
    #[error("communication failed")]
    Comm,
}

pub trait Display {
    fn show_environment(&mut self, reading: EnvironmentReading) -> Result<(), DisplayError>;
    fn clear(&mut self) -> Result<(), DisplayError> {
        Ok(())
    }
}
