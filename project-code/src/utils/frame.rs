use super::pixel::Pixel;
use super::image::Image;
use super::gpio_driver::ROWS;
use super::gpio_driver::COLUMNS;

pub struct Frame {
    pos: usize,
    pub pixels: Vec<Vec<Pixel>>,
}
impl Frame {
    pub fn new() -> Frame {
        let mut frame: Frame = Frame {
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
}