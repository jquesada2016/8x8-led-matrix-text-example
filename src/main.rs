use anyhow::Result;
use embedded_graphics::{
    mono_font::ascii::FONT_5X8, mono_font::MonoTextStyle, pixelcolor::BinaryColor, prelude::*,
    primitives as p, text::Text,
};
use rppal::gpio::{Gpio, Level, OutputPin};
use std::{sync::mpsc::channel, thread, time::Duration};

// Row pin numbers
const ROW_1: u8 = 8;
const ROW_2: u8 = 13;
const ROW_3: u8 = 7;
const ROW_4: u8 = 11;
const ROW_5: u8 = 0;
const ROW_6: u8 = 6;
const ROW_7: u8 = 1;
const ROW_8: u8 = 4;

// Column pin numbers
const COL_1: u8 = 16;
const COL_2: u8 = 2;
const COL_3: u8 = 3;
const COL_4: u8 = 9;
const COL_5: u8 = 5;
const COL_6: u8 = 10;
const COL_7: u8 = 14;
const COL_8: u8 = 15;

fn main() -> Result<()> {
    // Channel used to send time tick messages to the thread where the drawing
    // will take place.
    let (tx, rx) = channel();

    // We are using a new thread because we need to sleep on the main thread in
    // order to animate the text scrolling
    thread::spawn(move || {
        let gpio = Gpio::new().unwrap();

        let mut display = LedMatrix::new(
            &gpio, ROW_1, ROW_2, ROW_3, ROW_4, ROW_5, ROW_6, ROW_7, ROW_8, COL_1, COL_2, COL_3,
            COL_4, COL_5, COL_6, COL_7, COL_8,
        )
        .unwrap();

        // Used to calculate the transition for the animation
        let mut offset_x = 0u8;

        let character_style = MonoTextStyle::new(&FONT_5X8, true.into());

        let text = Text::new("I bet you can't do this!", (0, 7).into(), character_style);

        loop {
            // If we get a frame tick, then we increment the offset
            if rx.try_recv().is_ok() {
                offset_x = offset_x.wrapping_add(1);
            }

            let offset_x = offset_x as u32 % text.bounding_box().size.width;

            text.translate(Point::new(-(offset_x as i32), 0))
                .draw(&mut display)
                .unwrap();
        }
    });

    loop {
        // Sleep until the next frame should be rendered, in this case, 5 frames per second
        thread::sleep(Duration::from_millis(1000 / 5));

        // Notify the drawing thread that the next frame transition should be rendered
        tx.send(())?;
    }
}

struct LedMatrix {
    row_1: OutputPin,
    row_2: OutputPin,
    row_3: OutputPin,
    row_4: OutputPin,
    row_5: OutputPin,
    row_6: OutputPin,
    row_7: OutputPin,
    row_8: OutputPin,
    col_1: OutputPin,
    col_2: OutputPin,
    col_3: OutputPin,
    col_4: OutputPin,
    col_5: OutputPin,
    col_6: OutputPin,
    col_7: OutputPin,
    col_8: OutputPin,
}

impl OriginDimensions for LedMatrix {
    fn size(&self) -> Size {
        Size::new(8, 8)
    }
}

impl DrawTarget for LedMatrix {
    type Color = BinaryColor;
    type Error = std::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        // Draw each new pixel
        for Pixel(p, c) in pixels {
            // Only draw the pixel if it fits inside the 8x8 matrix
            if self.bounding_box().contains(p) {
                // Convert the pixel color into a voltage for the LED
                let level = match c {
                    BinaryColor::On => Level::High,
                    BinaryColor::Off => Level::Low,
                };

                // Because the Raspberry Pi 4 seems to not be able to drive the LEDs
                // without creating hot spots (some LED's brighter than others),
                // we want to light up only 1 LED at a time and at 50% brightness.
                // We do this by a simple software PWM with a period of 10us and a
                // duty cycle of 50%

                // Turn on the LED
                match p.x {
                    0 => self.col_1.write(!level),
                    1 => self.col_2.write(!level),
                    2 => self.col_3.write(!level),
                    3 => self.col_4.write(!level),
                    4 => self.col_5.write(!level),
                    5 => self.col_6.write(!level),
                    6 => self.col_7.write(!level),
                    7 => self.col_8.write(!level),
                    _ => unreachable!(),
                }
                match p.y {
                    0 => self.row_1.write(level),
                    1 => self.row_2.write(level),
                    2 => self.row_3.write(level),
                    3 => self.row_4.write(level),
                    4 => self.row_5.write(level),
                    5 => self.row_6.write(level),
                    6 => self.row_7.write(level),
                    7 => self.row_8.write(level),
                    _ => unreachable!(),
                }

                std::thread::sleep(std::time::Duration::from_micros(5));

                // Turn off the LED
                match p.x {
                    0 => self.col_1.write(level),
                    1 => self.col_2.write(level),
                    2 => self.col_3.write(level),
                    3 => self.col_4.write(level),
                    4 => self.col_5.write(level),
                    5 => self.col_6.write(level),
                    6 => self.col_7.write(level),
                    7 => self.col_8.write(level),
                    _ => unreachable!(),
                }
                match p.y {
                    0 => self.row_1.write(!level),
                    1 => self.row_2.write(!level),
                    2 => self.row_3.write(!level),
                    3 => self.row_4.write(!level),
                    4 => self.row_5.write(!level),
                    5 => self.row_6.write(!level),
                    6 => self.row_7.write(!level),
                    7 => self.row_8.write(!level),
                    _ => unreachable!(),
                }

                std::thread::sleep(std::time::Duration::from_micros(5));
            }
        }

        Ok(())
    }
}

impl LedMatrix {
    #[allow(clippy::too_many_arguments)]
    fn new(
        gpio: &Gpio,
        row_1_pin_number: u8,
        row_2_pin_number: u8,
        row_3_pin_number: u8,
        row_4_pin_number: u8,
        row_5_pin_number: u8,
        row_6_pin_number: u8,
        row_7_pin_number: u8,
        row_8_pin_number: u8,
        col_1_pin_number: u8,
        col_2_pin_number: u8,
        col_3_pin_number: u8,
        col_4_pin_number: u8,
        col_5_pin_number: u8,
        col_6_pin_number: u8,
        col_7_pin_number: u8,
        col_8_pin_number: u8,
    ) -> Result<Self> {
        let row_1 = gpio.get(row_1_pin_number)?.into_output_low();
        let row_2 = gpio.get(row_2_pin_number)?.into_output_low();
        let row_3 = gpio.get(row_3_pin_number)?.into_output_low();
        let row_4 = gpio.get(row_4_pin_number)?.into_output_low();
        let row_5 = gpio.get(row_5_pin_number)?.into_output_low();
        let row_6 = gpio.get(row_6_pin_number)?.into_output_low();
        let row_7 = gpio.get(row_7_pin_number)?.into_output_low();
        let row_8 = gpio.get(row_8_pin_number)?.into_output_low();

        let col_1 = gpio.get(col_1_pin_number)?.into_output_high();
        let col_2 = gpio.get(col_2_pin_number)?.into_output_high();
        let col_3 = gpio.get(col_3_pin_number)?.into_output_high();
        let col_4 = gpio.get(col_4_pin_number)?.into_output_high();
        let col_5 = gpio.get(col_5_pin_number)?.into_output_high();
        let col_6 = gpio.get(col_6_pin_number)?.into_output_high();
        let col_7 = gpio.get(col_7_pin_number)?.into_output_high();
        let col_8 = gpio.get(col_8_pin_number)?.into_output_high();

        Ok(Self {
            row_1,
            row_2,
            row_3,
            row_4,
            row_5,
            row_6,
            row_7,
            row_8,
            col_1,
            col_2,
            col_3,
            col_4,
            col_5,
            col_6,
            col_7,
            col_8,
        })
    }
}
