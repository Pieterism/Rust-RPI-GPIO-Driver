use COLOR_DEPTH;

#[derive(Copy, Clone, Debug)]
pub struct Pixel {
    pub r: u16,
    pub g: u16,
    pub b: u16,
}

impl Pixel {
    pub fn new() -> Pixel {
        let mut pixel: Pixel = Pixel {
            r: 0,
            g: 0,
            b: 0,
        };
        pixel
    }

    pub fn toFullColor(self: &mut Pixel){
        self.r = self.r * ((1<< COLOR_DEPTH)-1)/255;
        self.g = self.g * ((1<< COLOR_DEPTH)-1)/255;
        self.b = self.b * ((1<< COLOR_DEPTH)-1)/255;
    }
}