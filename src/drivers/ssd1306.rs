use display_interface::DisplayError;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use embedded_hal::i2c::I2c;
use ssd1306::I2CDisplayInterface;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::{DisplayConfig, DisplayRotation};
use ssd1306::size::DisplaySize128x64;

use display_interface_i2c::I2CInterface;

pub struct Ssd1306<I2C>
where
    I2C: I2c,
{
    inner: ssd1306::Ssd1306<
        I2CInterface<I2C>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,
}

impl<I2C, E> Ssd1306<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C) -> Result<Self, DisplayError> {
        let interface = I2CDisplayInterface::new(i2c);
        let mut driver =
            ssd1306::Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
                .into_buffered_graphics_mode();
        driver.init()?;
        driver.flush()?;
        Ok(Self { inner: driver })
    }

    pub fn show_text(&mut self, text: &str) -> Result<(), DisplayError> {
        self.inner.clear_buffer();
        let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        Text::with_baseline(text, Point::zero(), style, Baseline::Top).draw(&mut self.inner)?;
        self.inner.flush()
    }

    pub fn show_sensor_data(
        &mut self,
        temperature: f64,
        humidity: f64,
    ) -> Result<(), DisplayError> {
        self.inner.clear_buffer();

        let title_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let value_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

        let mut buf1 = [0u8; 64];
        let mut buf2 = [0u8; 64];

        Text::with_baseline("Temperature:", Point::new(0, 0), title_style, Baseline::Top)
            .draw(&mut self.inner)?;

        Text::with_baseline(
            format_no_std::show(&mut buf1, format_args!("{:.2} C", temperature)).unwrap(),
            Point::new(0, 12),
            value_style,
            Baseline::Top,
        )
        .draw(&mut self.inner)?;

        Text::with_baseline("Humidity:", Point::new(0, 32), title_style, Baseline::Top)
            .draw(&mut self.inner)?;

        Text::with_baseline(
            format_no_std::show(&mut buf2, format_args!("{:.2} %", humidity)).unwrap(),
            Point::new(0, 44),
            value_style,
            Baseline::Top,
        )
        .draw(&mut self.inner)?;

        self.inner.flush()
    }
}
