use super::pixel::Pixel;
use super::gpio_driver::{COLUMNS, ROWS};

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Vec<Pixel>>
}

impl Image {
    pub fn new() -> Image {
        let mut image: Image = Image {
            width: COLUMNS as u32,
            height: ROWS as u32,
            pixels: vec![vec![Pixel::new(); COLUMNS as usize]; ROWS as usize],
        };
        image
    }
}
