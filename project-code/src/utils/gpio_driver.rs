
use mmap::{MemoryMap, MapOption};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::error::Error;
use std::os::unix::io::AsRawFd;
use super::image::Image;
use super::frame::Frame;
use super::pixel::Pixel;
use super::time::Timer;

//necessary for running on RaspPi
use std;
use time;
use libc;


const PIN_OE: u64 = 4;
const PIN_CLK: u64 = 17;
const PIN_LAT: u64 = 21;
const PIN_A: u64 = 22;
const PIN_B: u64 = 26;
const PIN_C: u64 = 27;
const PIN_D: u64 = 20;
const PIN_E: u64 = 24;
const PIN_R1: u64 = 5;
const PIN_G1: u64 = 13;
const PIN_B1: u64 = 6;
const PIN_R2: u64 = 12;
const PIN_G2: u64 = 16;
const PIN_B2: u64 = 23;

const BCM2709_PERI_BASE: u64 = 0x3F000000;
const GPIO_REGISTER_OFFSET: u64 = 0x200000;

pub const REGISTER_BLOCK_SIZE: u64 = 4096;
pub const COLOR_DEPTH: usize = 8;
pub const ROWS: usize = 16;
pub const COLUMNS: usize = 32;
pub const SUB_PANELS: usize = 2;

macro_rules! GPIO_BIT {
    ($bit:expr) => {
        1 << $bit
    };
}

const VALID_BITS: u64 = GPIO_BIT!(PIN_OE) | GPIO_BIT!(PIN_CLK) | GPIO_BIT!(PIN_LAT) |
    GPIO_BIT!(PIN_A) | GPIO_BIT!(PIN_B) | GPIO_BIT!(PIN_C) | GPIO_BIT!(PIN_D) | GPIO_BIT!(PIN_E) |
    GPIO_BIT!(PIN_R1) | GPIO_BIT!(PIN_G1) | GPIO_BIT!(PIN_B1) |
    GPIO_BIT!(PIN_R2) | GPIO_BIT!(PIN_G2) | GPIO_BIT!(PIN_B2);

pub struct GPIO {
    gpio_map_: Option<MemoryMap>,
    output_bits_: u32,
    input_bits_: u32,
    slowdown_: u32,
    gpio_port_: *mut u32,
    gpio_set_bits_: *mut u32,
    gpio_clr_bits_: *mut u32,
    gpio_read_bits_: *mut u32,
    pub row_mask: u32,
    bitplane_timings: [u32; COLOR_DEPTH],
}

impl GPIO {
    fn configure_output_pin(self: &mut GPIO, pin_num: u64) {
        let register_num = (pin_num / 10) as isize;
        let register_ref = unsafe { self.gpio_port_.offset(register_num) };
        let current_val = unsafe { std::ptr::read_volatile(register_ref) };
        let new_val = (current_val & !(7 << ((pin_num % 10) * 3))) | (1 << ((pin_num % 10) * 3));
        unsafe { std::ptr::write_volatile(register_ref, new_val) };
    }

    fn init_outputs(self: &mut GPIO, mut outputs: u32) -> u32 {
        outputs &= VALID_BITS as u32;
        outputs &= !(self.output_bits_ | self.input_bits_);

        for b in 0..28 {
            if GPIO_BIT!(b) & outputs != 0 {
                self.configure_output_pin(b as u64);
            }
        }
        self.output_bits_ |= outputs;

        outputs
    }

    fn set_bits(self: &mut GPIO, value: u32) {
        unsafe {
            std::ptr::write_volatile(self.gpio_set_bits_, value);
            for i in 0..self.slowdown_ {
                std::ptr::write_volatile(self.gpio_set_bits_, value);
            }
        }
    }

    fn clear_bits(self: &mut GPIO, value: u32) {
        unsafe {
            std::ptr::write_volatile(self.gpio_clr_bits_, value);
            for i in 0..self.slowdown_ {
                std::ptr::write_volatile(self.gpio_clr_bits_, value);
            }
        }
    }

    fn write_masked_bits(self: &mut GPIO, value: u32, mask: u32) {
        self.clear_bits(!value & mask);
        self.set_bits(value & mask);
    }

    pub fn new(slowdown: u32) -> GPIO {
        let map = mmap_bcm_register(GPIO_REGISTER_OFFSET as usize);

        if map.is_none() {
            eprint!("map error");
        }

        let mut io: GPIO = GPIO {
            gpio_map_: None,
            output_bits_: 0,
            input_bits_: 0,
            slowdown_: slowdown,
            gpio_port_: 0 as *mut u32,
            gpio_set_bits_: 0 as *mut u32,
            gpio_clr_bits_: 0 as *mut u32,
            gpio_read_bits_: 0 as *mut u32,
            row_mask: 0,
            bitplane_timings: [0; COLOR_DEPTH],
        };

        match &map {
            &Some(ref m) => {
                unsafe {
                    io.gpio_port_ = m.data() as *mut u32;
                    io.gpio_set_bits_ = io.gpio_port_.offset(7);
                    io.gpio_clr_bits_ = io.gpio_port_.offset(10);
                    io.gpio_read_bits_ = io.gpio_port_.offset(13);
                }
                let mut all_used_bits: u32 = 0;
                all_used_bits |= GPIO_BIT!(PIN_OE) | GPIO_BIT!(PIN_CLK) | GPIO_BIT!(PIN_LAT) |
                    GPIO_BIT!(PIN_R1) | GPIO_BIT!(PIN_G1) | GPIO_BIT!(PIN_B1) |
                    GPIO_BIT!(PIN_R2) | GPIO_BIT!(PIN_G2) | GPIO_BIT!(PIN_B2);

                set_row_mask(&mut io);

                all_used_bits |= io.row_mask;
                let result = io.init_outputs(all_used_bits);
                assert_eq!(result, all_used_bits);

                let mut timing_ns: u32 = 1000;
                for b in 0..COLOR_DEPTH {
                    io.bitplane_timings[b] = timing_ns;
                    timing_ns *= 2;
                }
            }
            &None => {}
        }
        io.gpio_map_ = map;
        io
    }

    pub fn render_frame(&mut self, interrupt_received: Arc<AtomicBool>, image: &Image, frame: &mut Frame, timer: &Timer, scrolling: bool) {
        let mut prev_frame_time = time::get_time();
        let mut current_time = time::get_time();

        while interrupt_received.load(Ordering::SeqCst) == false {
            for row_counter in 0..ROWS / 2 {
                for bitplane_counter in 0..COLOR_DEPTH {
                    self.send_values( &timer, &frame, row_counter, bitplane_counter);
                };
            };

            current_time = time::get_time();
            let difference = current_time - prev_frame_time;
            if scrolling && difference >= time::Duration::milliseconds(10) {
                frame.next_image_frame(&image);
                prev_frame_time = current_time;
            };
        };
        if interrupt_received.load(Ordering::SeqCst) == true {
            println!("Received CTRL-C");
        } else {
            println!("Timeout reached");
        };
        self.set_bits(GPIO_BIT!(PIN_OE));
    }

    fn send_values(&mut self, timer: &Timer, frame: &Frame, row: usize, bitplane_counter: usize) {
        let row_mask = self.row_mask;
        let color_clock_mask = GPIO_BIT!(PIN_R1) | GPIO_BIT!(PIN_G1) | GPIO_BIT!(PIN_B1) | GPIO_BIT!(PIN_R2) | GPIO_BIT!(PIN_G2) | GPIO_BIT!(PIN_B2) | GPIO_BIT!(PIN_CLK);

        for c in 0..32 {
            self.clear_bits(color_clock_mask);
            let pixel_top = frame.pixels[row][c];
            let pixel_bot = frame.pixels[ROWS / 2 + row][c];
            let plane_bits = get_plane_bits(pixel_top, pixel_bot, bitplane_counter);

            self.write_masked_bits(plane_bits, color_clock_mask);
            self.set_bits(GPIO_BIT!(PIN_CLK));
        };

        self.clear_bits(GPIO_BIT!(PIN_R1) | GPIO_BIT!(PIN_G1) | GPIO_BIT!(PIN_B1) | GPIO_BIT!(PIN_R2) | GPIO_BIT!(PIN_G2) | GPIO_BIT!(PIN_B2) | GPIO_BIT!(PIN_CLK));
        self.write_masked_bits(get_row_bits(row), row_mask);

        self.set_bits(GPIO_BIT!(PIN_LAT));
        self.clear_bits(GPIO_BIT!(PIN_LAT));
        self.clear_bits(GPIO_BIT!(PIN_OE));
        timer.nanosleep(self.bitplane_timings[bitplane_counter] as u32);
        self.set_bits(GPIO_BIT!(PIN_OE));
    }

}

pub fn mmap_bcm_register(register_offset: usize) -> Option<MemoryMap> {
    eprintln!("starting bcm register");

    let mem_file =
        match OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_SYNC)
            .open("/dev/mem") {
            Err(why) => panic!("couldn't open /dev/mem: {}", why.description()),
            Ok(file) => file
        };

    let mmap_options = &[
        MapOption::MapNonStandardFlags(libc::MAP_SHARED),
        MapOption::MapReadable,
        MapOption::MapWritable,
        MapOption::MapFd(mem_file.as_raw_fd()),
        MapOption::MapOffset(BCM2709_PERI_BASE as usize + register_offset as usize)
    ];

    let result = match MemoryMap::new(REGISTER_BLOCK_SIZE as usize, mmap_options)
        {
            Ok(mmap) => {
                println!("Successfully created the mmap: {}", mmap.len());
                mmap
            }
            Err(err) => panic!("Could not read the mmap: {}", err),
        };


    return match result.data().is_null() {
        true => {
            eprintln!("mmap error: {}", std::io::Error::last_os_error());
            eprintln!("Pi3: MMapping from base 0x{:X}, offset 0x{:X}", BCM2709_PERI_BASE, register_offset);
            None
        }
        false => Some(result)
    };
}

fn get_plane_bits(top: Pixel, bot: Pixel, plane: usize) -> u32 {
    let mut out: u32 = 0;
    if top.r & (1 << plane) != 0 {
        out |= GPIO_BIT!(PIN_R1);
    }
    if  bot.r & (1 << plane) != 0 {
        out |= GPIO_BIT!(PIN_R2);
    }
    if top.b & (1 << plane) != 0 {
        out |= GPIO_BIT!(PIN_B1);
    }
    if bot.b & (1 << plane) != 0 {
        out |= GPIO_BIT!(PIN_B2);
    }
    if top.g & (1 << plane) != 0 {
        out |= GPIO_BIT!(PIN_G1);
    }
    if bot.g & (1 << plane) != 0 {
        out |= GPIO_BIT!(PIN_G2);
    }
    out
}

fn get_row_bits(double_row: usize) -> u32 {

    let mut pin = 0;
    if double_row & 0x01 != 0 {
        pin |= GPIO_BIT!(PIN_A);
    }
    if double_row & 0x02 != 0 {
        pin |= GPIO_BIT!(PIN_B);
    }
    if double_row & 0x04 != 0 {
        pin |= GPIO_BIT!(PIN_C);
    }
    pin
}

fn set_row_mask(gpio: &mut GPIO) {
    gpio.row_mask = GPIO_BIT!(PIN_A);
    let rows_count = ROWS / SUB_PANELS;
    if rows_count > 2 {
        gpio.row_mask |= GPIO_BIT!(PIN_B);
    }
    if rows_count > 4 {
        gpio.row_mask |= GPIO_BIT!(PIN_C);
    }
    if rows_count > 8 {
        gpio.row_mask |= GPIO_BIT!(PIN_D);
    }
    if rows_count > 16 {
        gpio.row_mask |= GPIO_BIT!(PIN_E);
    }
}

#[test]
fn get_row_bits_test() {
    assert_eq!(0                                                            , get_row_bits(0), "Invalid row bits");
    assert_eq!(GPIO_BIT!(PIN_A)                                             , get_row_bits(1), "Invalid row bits");
    assert_eq!(GPIO_BIT!(PIN_B)                                             , get_row_bits(2), "Invalid row bits");
    assert_eq!(GPIO_BIT!(PIN_A) | GPIO_BIT!(PIN_B)                          , get_row_bits(3), "Invalid row bits");
    assert_eq!(GPIO_BIT!(PIN_C)                                             , get_row_bits(4), "Invalid row bits");
    assert_eq!(GPIO_BIT!(PIN_C) | GPIO_BIT!(PIN_A)                          , get_row_bits(5), "Invalid row bits");
    assert_eq!(GPIO_BIT!(PIN_C) | GPIO_BIT!(PIN_B)                          , get_row_bits(6), "Invalid row bits");
    assert_eq!(GPIO_BIT!(PIN_C) | GPIO_BIT!(PIN_B) | GPIO_BIT!(PIN_A)       , get_row_bits(7), "Invalid row bits");

}