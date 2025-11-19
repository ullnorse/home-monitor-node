use embedded_hal::i2c::I2c;
use ssd1306::{
    mode::BufferedGraphicsMode,
    prelude::{DisplayConfig, DisplayRotation},
    size::DisplaySize128x64,
    I2CDisplayInterface,
    Ssd1306,
};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

use display_interface_i2c::I2CInterface;

use crate::error::AppError;

pub struct Display<I2C>
where
    I2C: I2c,
{
    driver: Ssd1306<
        I2CInterface<I2C>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >
}

impl<I2C, E> Display<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C) -> Result<Self, AppError> {
        let interface = I2CDisplayInterface::new(i2c);
        let mut driver =
            Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_buffered_graphics_mode();
        driver.init().map_err(|_| AppError::DisplayInit)?;
        //driver.clear();
        driver.flush().map_err(|_| AppError::DisplayInit)?;
        Ok(Self { driver })
    }

    pub fn show_text(&mut self, text: &str) -> Result<(), AppError> {
        self.driver.clear_buffer();
        let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        Text::with_baseline(text, Point::zero(), style, Baseline::Top)
            .draw(&mut self.driver)
            .map_err(|_| AppError::DisplayDraw)?;
        self.driver.flush().map_err(|_| AppError::DisplayDraw)
    }
}