use super::pixel::Pixel;

// This is a representation of the "raw" image
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Vec<Pixel>>
}

// TODO: Add your PPM parser here
// NOTE/WARNING: Please make sure that your implementation can handle comments in the PPM file
// You do not need to add support for any formats other than P6
// You may assume that the max_color value is always 255, but you should add sanity checks
// to safely reject files with other max_color values
impl Image {

}
