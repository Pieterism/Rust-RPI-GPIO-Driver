

#[macro_use] extern crate simple_error;

use std::error::Error;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Cursor};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fmt;
use std::io::prelude::*;
use std::io::{Seek, SeekFrom};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use shuteye::sleep;
use std::time::Duration;

#[derive(Clone)]
struct Pixel
{
    R: u32,
    G: u32,
    B: u32
}

struct Image
{
    width: u32,
    height: u32,
    pixels: Vec<Vec<Pixel>>
}

pub fn show_image(image: &Image)
{
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let display_mode = video_subsystem.current_display_mode(0).unwrap();

    let w = match display_mode.w as u32 > image.width {
        true => image.width,
        false => display_mode.w as u32
    };
    let h = match display_mode.h as u32 > image.height {
        true => image.height,
        false => display_mode.h as u32
    };

    let window = video_subsystem
        .window("Image", w, h)
        .build()
        .unwrap();
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .unwrap();
    let black = sdl2::pixels::Color::RGB(0, 0, 0);

    let mut event_pump = sdl.event_pump().unwrap();
    // render image
    canvas.set_draw_color(black);
    canvas.clear();

    for r in 0..image.height {
        for c in 0..image.width {
            let pixel = &image.pixels[image.height as usize - r as usize - 1][c as usize];
            canvas.set_draw_color(Color::RGB(pixel.R as u8, pixel.G as u8, pixel.B as u8));
            canvas.fill_rect(Rect::new(c as i32, r as i32, 1, 1)).unwrap();
        }
    }

    canvas.present();

    'main: loop
        {
            for event in event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit {..} => break 'main,
                    _ => {},
                }
            }

            sleep(Duration::new(0, 250000000));
        }

}

pub fn decode_ppm_image(cursor: &mut Cursor<Vec<u8>>) -> Result<Image, Box<std::error::Error>> {
    let mut image = Image {
        width: 0,
        height: 0,
        pixels: vec![]
    };

    read_constant(cursor : &mut Cursor<Vec<u8>>);


    Ok(image)
}

fn read_constant(cursor: &mut Cursor<Vec<u8>>) {
    let mut buffer :[u8, 2] = [0,0];
    cursor.read(cursor.into(), buffer);
    println!("{:?}", buffer);
}
