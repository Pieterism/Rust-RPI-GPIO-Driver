use super::pixel::Pixel;
use super::image::Image;
use super::file_reader::read_ppm_file;
use super::gpio_driver::ROWS;
use super::gpio_driver::COLUMNS;
use std::path::Path;

const BORDER_PIXEL: Pixel = Pixel::new_colored_pixel(230, 230, 230);
const GAME_OVER_IMG_PATH: &str = "resources/snake/game_over.ppm";

pub struct Frame {
    BORDER_PIXEL: Pixel,
    pos: usize,
    pub pixels: Vec<Vec<Pixel>>,
}

impl Frame {
    pub fn new() -> Frame {
        let mut frame: Frame = Frame {
            BORDER_PIXEL: Pixel::new_colored_pixel(230, 230, 230),
            pos: 0,
            pixels: vec![vec![Pixel::new(); COLUMNS as usize]; ROWS as usize],
        };
        frame
    }

    pub fn next_image_frame(self: &mut Frame, image: &Image) {
        for row in 0..ROWS {
            for col in 0..COLUMNS {
                let img_pos = (self.pos + col) % image.width as usize;

                self.pixels[row][col] = image.pixels[row][img_pos].clone();
            }
        }

        self.pos = self.pos + 1;
        if self.pos >= image.width as usize {
            self.pos = 0;
        }
    }

    pub fn draw_border(self: &mut Frame) {
        for row in 0..ROWS {
            for col in 0..COLUMNS {
                if row == 0 || row == ROWS || col == 0 || col == COLUMNS {
                    self.pixels[row][col] = self.BORDER_PIXEL;
                }
            }
        }
    }

    pub fn draw_game_over(self: &mut Frame) {
        next_image_frame(&read_ppm_file(&Path::new(GAME_OVER_IMG_PATH)));
    }
}