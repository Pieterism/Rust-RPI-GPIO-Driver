
//#[macro_use] extern crate simple_error;

use std::error::Error;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Cursor};
use std::fmt;
use std::io::prelude::*;
use std::io::{Seek, SeekFrom};
use shuteye::sleep;
use std::time::Duration;
use std::num::ParseIntError;
use super::pixel::Pixel as Pixel;
use super::image::Image;
// use sdl2::pixels::Color;
// use sdl2::rect::Rect;

/*
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
            canvas.set_draw_color(Color::RGB(pixel.r as u8, pixel.g as u8, pixel.b as u8));
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
*/

pub fn read_ppm_file(path: &Path) -> Image {
    let display = path.display();

    let mut file = match File::open(&path)    {
        Err(why) => panic!("Could not open file: {} (Reason: {})",
                           display, why.description()),
        Ok(file) => file
    };

    let mut raw_file = Vec::new();
    file.read_to_end(&mut raw_file).unwrap();

    let mut cursor = Cursor::new(raw_file);
    let mut image = match decode_ppm_image(&mut cursor) {
        Ok(img) => img,
        Err(why) => panic!("Could not parse PPM file - Desc: {}", why.description()),
    };

    image
}

fn decode_ppm_image(cursor: &mut Cursor<Vec<u8>>) -> Result<Image, ParseIntError> {
    let mut image = Image {
        width: 0,
        height: 0,
        pixels: vec![]
    };

    read_constants(cursor);
    read_char(cursor);

    image.width = read_size_propertie(cursor)?;
    image.height = read_size_propertie(cursor)?;
    let max_size = read_size_propertie(cursor)?;

    println!("{}", image.width);
    println!("{}", image.height);
    println!("{}", max_size);

    let mut pixels: Vec<Vec<Pixel>> = Vec::new();
    let mut counter: usize = 0;

    if max_size <= 255 {
        for length in 0..image.height{
            let mut row: Vec<Pixel> = Vec::new();
            for width in 0..image.width {
                let pixel = build_pixel_u8(cursor);
                counter+= 1;
                row.push(pixel);
            };
            pixels.push(row);
        };
    } else {
        for length in 0..image.height{
            let mut row: Vec<Pixel> = Vec::new();
            for width in 0..image.width {
                let pixel = build_pixel_u16(cursor);
                row.push(pixel);
                counter+= 1;
            };
            pixels.push(row);
        };
    };

    image.pixels = pixels;
    Ok(image)
}

fn build_pixel_u16(cursor: &mut Cursor<Vec<u8>>) -> Pixel {
    Pixel {
        r: read_u16(cursor),
        g: read_u16(cursor),
        b: read_u16(cursor)
    }
}

fn build_pixel_u8(cursor: &mut Cursor<Vec<u8>>) -> Pixel {
    Pixel {
        r: read_u8(cursor) as u16,
        g: read_u8(cursor) as u16,
        b: read_u8(cursor) as u16
    }
}

fn read_u16(cursor: &mut Cursor<Vec<u8>>) -> u16 {
    let mut buffer: [u8; 2] = [0,0];
    cursor.read(&mut buffer);
    let result_string: String = buffer.to_vec().into_iter()
        .map(|value| value as char)
        .collect();
    result_string.parse::<u16>().unwrap()
}

fn read_u8(cursor: &mut Cursor<Vec<u8>>) -> u8{
    let mut buffer: [u8; 1] = [0];
    cursor.read(&mut buffer);
    buffer[0]
}

fn read_size_propertie(cursor: &mut Cursor<Vec<u8>>) -> Result<u32, ParseIntError> {
    let mut result_buffer: Vec<char> = Vec::new();
    loop {
        let value = read_char(cursor);
        if value == ' ' || value == '\n' || value == '\t' {
            break;
        }
        result_buffer.push(value);
    }
    let result_string: String = result_buffer.into_iter().collect();
    result_string.parse::<u32>()
}

fn read_char(cursor: &mut Cursor<Vec<u8>>) -> char{
    let mut buffer: [u8; 1] = [0];
    cursor.read(&mut buffer);
    buffer[0] as char
}

fn read_constants(cursor: &mut Cursor<Vec<u8>>){
    let mut buffer :[u8; 2] = [0,0];
    cursor.read( &mut buffer);
    assert_eq!(buffer[0], 'P' as u8, "Invalid header");
    assert_eq!(buffer[1], '6' as u8, "Invalid PNM type");
    println!("P6");
}


#[test]
fn decode_ppm_image_test() {
    let mut vector: Vec<u8> = vec!['P' as u8, '6' as u8, '\n' as u8, '3' as u8, '2' as u8, ' ' as u8, '1' as u8, '6' as u8, '\n' as u8];
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(vector);
    decode_ppm_image(&mut cursor);
}

#[test]
fn read_size_properties_test_new_line_separated() {
    let mut vector: Vec<u8> = vec!['3' as u8, '2' as u8, '\n' as u8];
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(vector);
    let result = read_size_propertie(&mut cursor);
    const expected: u32 = 32;
    assert_eq!(expected, result.unwrap(), "Did not get the expected value back");
}

#[test]
fn read_size_properties_test_blank_space_separated() {
    let mut vector: Vec<u8> = vec!['3' as u8, '2' as u8, ' ' as u8];
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(vector);

    let result = read_size_propertie(&mut cursor);
    let expected: u32 = 32;
    assert_eq!(expected, result.unwrap(), "Did not get the expected value back");
}

#[test]
fn sample_file_test() {

}