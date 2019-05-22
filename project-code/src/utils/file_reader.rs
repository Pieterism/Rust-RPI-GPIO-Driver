use std::error::Error;
use std::fs::File;
use std::io::{Cursor, Read};
use std::num::ParseIntError;
use std::path::Path;

use super::image::Image;
use super::pixel::Pixel as Pixel;

pub fn read_ppm_file(path: &Path) -> Image {
    let display = path.display();

    let mut file = match File::open(&path)    {
        Err(why) => panic!("Could not open file: {} (Reason: {})",display, why.description()),
        Ok(file) => file
    };

    let mut raw_file = Vec::new();
    file.read_to_end(&mut raw_file).unwrap();
    let mut cursor = Cursor::new(raw_file);

    let image = match decode_ppm_image(&mut cursor) {
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

    println!("Decode started");

    read_constants(cursor);

    read_size_or_comment(cursor, &mut image);
    println!("{}", image.width);
    println!("{}", image.height);

    let max_size = read_max_value_or_comment(cursor)?;

    let mut pixels: Vec<Vec<Pixel>> = Vec::new();

    if max_size <= 255 {
        for _length in 0..image.height{
            let mut row: Vec<Pixel> = Vec::new();
            for _width in 0..image.width {
                let pixel = build_pixel_u8(cursor);
                row.push(pixel);
            };
            pixels.push(row);
        };
    } else {
        for _length in 0..image.height{
            let mut row: Vec<Pixel> = Vec::new();
            for _width in 0..image.width {
                let pixel = build_pixel_u16(cursor);
                row.push(pixel);
            };
            pixels.push(row);
        };
    };

    image.pixels = pixels;
    Ok(image)
}

fn read_size_or_comment(cursor: &mut Cursor<Vec<u8>>, image: &mut Image) {
    let first_character = read_char(cursor);
    if first_character == '#' {
        read_line(cursor);
        read_size_or_comment(cursor, image);
    }
    else {
        image.width = read_size_propertie(cursor, Option::Some(first_character)).unwrap();
        image.height = read_size_propertie(cursor, Option::None).unwrap();
    }
}

fn read_max_value_or_comment(cursor: &mut Cursor<Vec<u8>>) -> Result<u32, ParseIntError> {
    let first_character = read_char(cursor);
    if first_character == '#' {
        read_line(cursor);
        return read_max_value_or_comment(cursor);
    }
    read_size_propertie(cursor, Option::Some(first_character))
}

fn read_line(cursor: &mut Cursor<Vec<u8>>) {
    let mut result_buffer: Vec<char> = Vec::new();
    loop {
        let value = read_char(cursor);
        result_buffer.push(value);
        if value == '\n' || value == '\r' {
            break;
        }
    }
}

fn read_size_propertie(cursor: &mut Cursor<Vec<u8>>, first_character: Option<char>) -> Result<u32, ParseIntError> {
    let mut result_buffer: Vec<char> = Vec::new();
    if first_character.is_some() {
        result_buffer.push(first_character.unwrap());
    }
    read_until_split_character(cursor, &mut result_buffer);
    let result_string: String = result_buffer.into_iter().collect();
    result_string.parse::<u32>()
}

fn read_until_split_character(cursor: &mut Cursor<Vec<u8>>, result_buffer: &mut Vec<char>) {
    loop {
        let value = read_char(cursor);
        let condition = value == ' ' || value == '\n' || value == '\t' || value == '\r';
        if condition {
            break;
        };
        result_buffer.push(value);
    };
}

fn read_char(cursor: &mut Cursor<Vec<u8>>) -> char{
    let mut buffer: [u8; 1] = [0];
    match cursor.read(&mut buffer) {
        Ok(buffer) => buffer,
        Err(_err) => panic!("Reading of characters failed!")
    };
    buffer[0] as char
}

fn read_constants(cursor: &mut Cursor<Vec<u8>>){
    assert_eq!(read_char(cursor), 'P', "Invalid header");
    assert_eq!(read_char(cursor), '6', "Invalid PPM type");
    read_char(cursor);
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
    match cursor.read(&mut buffer) {
        Ok(buffer) => buffer,
        Err(_err) => panic!("Reading of u16 failed!")
    };
    let result_string: String = buffer.to_vec().into_iter()
        .map(|value| value as char)
        .collect();
    result_string.parse::<u16>().unwrap()
}

fn read_u8(cursor: &mut Cursor<Vec<u8>>) -> u8{
    let mut buffer: [u8; 1] = [0];
    match cursor.read(&mut buffer) {
        Ok(buffer) => buffer,
        Err(_err) => panic!("Reading of u8 failed!")
    };
    buffer[0]
}


#[test]
fn read_file_header_test_P_values() {
    let vector: Vec<u8> = vec!['P' as u8, '6' as u8, '\n' as u8];
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(vector);
    read_constants(&mut cursor);
}

#[test]
fn read_file_header_test_width_and_height_no_comments() {
    let vector: Vec<u8> = vec!['3' as u8, '2' as u8, ' ' as u8, '1' as u8, '6' as u8, '\n' as u8];
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(vector);
    let mut image : Image = Image::new();
    read_size_or_comment(&mut cursor, &mut image);

    assert_eq!(32, image.width ,"Image width is not 32");
    assert_eq!(16, image.height ,"Image height is not 16");
}

#[test]
fn read_file_header_test_width_and_height_with_comments() {
    let vector: Vec<u8> = vec!['#' as u8, 'M' as u8, 'y' as u8, ' ' as u8, 'c' as u8, 'o' as u8, 'm' as u8, 'm' as u8, 'e' as u8, 'n' as u8, 't' as u8, '\n' as u8,
        '3' as u8, '2' as u8, ' ' as u8, '1' as u8, '6' as u8, '\n' as u8];
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(vector);
    let mut image : Image = Image::new();
    read_size_or_comment(&mut cursor, &mut image);

    assert_eq!(32, image.width ,"Image width is not 32");
    assert_eq!(16, image.height ,"Image height is not 16");
}

#[test]
fn read_file_header_test_max_value_no_comments() {
    let vector: Vec<u8> = vec!['2' as u8, '5' as u8, '5' as u8, '\n' as u8];
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(vector);
    let mut image : Image = Image::new();
    let result = read_max_value_or_comment(&mut cursor);

    assert_eq!(255, result.unwrap() ,"The right max value was not found");
}

#[test]
fn read_file_header_test_max_value_with_comments() {
    let vector: Vec<u8> = vec!['#' as u8, 'M' as u8, 'y' as u8, ' ' as u8, 'c' as u8, 'o' as u8, 'm' as u8, 'm' as u8, 'e' as u8, 'n' as u8, 't' as u8, '\n' as u8,
                               '2' as u8, '5' as u8, '5' as u8, '\n' as u8];
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(vector);
    let mut image : Image = Image::new();
    let result = read_max_value_or_comment(&mut cursor);

    assert_eq!(255, result.unwrap() ,"The right max value was not found");
}

#[test]
fn read_file_header_integration_test_no_comments() {
    let path = Path::new("resources/testfile_no_comments.ppm");
    let image = read_ppm_file(&path);

    assert_eq!(32, image.width ,"Image width is not 32");
    assert_eq!(16, image.height ,"Image height is not 16");
}

#[test]
fn read_file_header_integration_test_with_comments() {
    let path = Path::new("resources/testfile_with_comments.ppm");
    let image = read_ppm_file(&path);

    assert_eq!(32, image.width ,"Image width is not 32");
    assert_eq!(16, image.height ,"Image height is not 16");
}
