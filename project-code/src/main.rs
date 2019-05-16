// Skeleton code for your Rust projects
// I added several comments and annotations to this file.
// _Please_ read them carefully. They are very important.
// The most important comments are all annotated with "NOTE/WARNING:"

// I will grade your code quality primarily on how "idiomatic" your Rust
// code is, and how well you implemented the "safe unsafety" guidelines.

//mods
mod utils;

#[macro_use]
extern crate simple_error;
extern crate libc;
extern crate time;
extern crate ctrlc;
extern crate shuteye;
extern crate mmap;
extern crate nix;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::os::unix::io::AsRawFd;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::error::Error;
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use std::time::Duration;
use shuteye::sleep;
use mmap::{MemoryMap, MapOption};
use utils::file_reader;
use utils::image::Image;
use utils::pixel::Pixel;

struct GPIO {
    gpio_map_: Option<MemoryMap>,
    output_bits_: u32,
    input_bits_: u32,
    slowdown_: u32,
    // Please refer to the GPIO_SetBits and GPIO_ClearBits functions in the reference implementation to see how this is used.
    gpio_port_: *mut u32,
    // A raw pointer that points to the base of the GPIO register file
    gpio_set_bits_: *mut u32,
    // A raw pointer that points to the pin output register (see section 2.1 in the assignment)
    gpio_clr_bits_: *mut u32,
    // A raw pointer that points to the pin output clear register (see section 2.1)
    gpio_read_bits_: *mut u32,
    // A raw pointer that points to the pin level register (see section 2.1)
    pub row_mask: u32,
    bitplane_timings: [u32; COLOR_DEPTH],
}

// This is a representation of the frame we're currently rendering
struct Frame {
    pos: usize,
    pixels: Vec<Vec<Pixel>>,
}

// Use this struct to implement high-precision nanosleeps
struct Timer {
    _timemap: Option<MemoryMap>,
    timereg: *mut u32, // a raw pointer to the 1Mhz timer register (see section 2.5 in the assignment)
}

// ============================================================================
// GPIO configuration parameters for the raspberry pi 3
// ============================================================================

const BCM2709_PERI_BASE: u64 = 0x3F000000;
const GPIO_REGISTER_OFFSET: u64 = 0x200000;
const TIMER_REGISTER_OFFSET: u64 = 0x3000;
const TIMER_OVERFLOW_VALUE: u32 = 4294967295;
const REGISTER_BLOCK_SIZE: u64 = 4096;
const COLOR_DEPTH: usize = 8;
const ROWS: usize = 16;
const COLUMNS: usize = 32;
const SUB_PANELS: usize = 2;

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

// Convenience macro for creating bitmasks. See comment above "impl GPIO" below
macro_rules! GPIO_BIT {
    ($bit:expr) => {
        1 << $bit
    };
}

// Use this bitmask for sanity checks
const VALID_BITS: u64 = GPIO_BIT!(PIN_OE) | GPIO_BIT!(PIN_CLK) | GPIO_BIT!(PIN_LAT) |
    GPIO_BIT!(PIN_A) | GPIO_BIT!(PIN_B) | GPIO_BIT!(PIN_C) | GPIO_BIT!(PIN_D) | GPIO_BIT!(PIN_E) |
    GPIO_BIT!(PIN_R1) | GPIO_BIT!(PIN_G1) | GPIO_BIT!(PIN_B1) |
    GPIO_BIT!(PIN_R2) | GPIO_BIT!(PIN_G2) | GPIO_BIT!(PIN_B2);

fn mmap_bcm_register(register_offset: usize) -> Option<MemoryMap> {
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

impl GPIO {
    //
    // configures pin number @pin_num as an output pin by writing to the
    // appropriate Function Select register (see section 2.1).
    //
    // NOTE/WARNING: This method configures one pin at a time. The @pin_num argument
    // that is expected here is really a pin number and not a bitmask!
    //
    // Doing something like:
    //     io.configure_output_pin(VALID_BITS);
    // Would be WRONG! This call would make the program crash.
    //
    // Doing something like:
    //     if GPIO_BIT!(PIN_A) & VALID_BITS {
    //         io.configure_output_pin(PIN_A);
    //     }
    // Would be OK!
    //
    fn configure_output_pin(self: &mut GPIO, pin_num: u64) {
        let register_num = (pin_num / 10) as isize;
        let register_ref = unsafe { self.gpio_port_.offset(register_num) };
        // NOTE/WARNING: When reading from or writing to MMIO memory regions, you MUST
        // use the std::ptr::read_volatile and std::ptr::write_volatile functions
        let current_val = unsafe { std::ptr::read_volatile(register_ref) };
        // the bit range within the register is [(pin_num % 10) * 3 .. (pin_num % 10) * 3 + 2]
        // we need to set these bits to 001
        let new_val = (current_val & !(7 << ((pin_num % 10) * 3))) | (1 << ((pin_num % 10) * 3));
        // NOTE/WARNING: When reading from or writing to MMIO memory regions, you MUST
        // use the std::ptr::read_volatile and std::ptr::write_volatile functions
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

    // Write all the bits of @value that also appear in @mask. Leave the rest untouched.
    // @value and @mask are bitmasks
    fn write_masked_bits(self: &mut GPIO, value: u32, mask: u32) {
        self.clear_bits(!value & mask);
        self.set_bits(value & mask);
    }

    /*    fn GPIO_Write(self: &mut GPIO, value: u32) {
            self.write_masked_bits(value, self.output_bits_);
        }*/

    /*    fn GPIO_Read(self: &mut GPIO) -> u32 {
            self.gpio_read_bits_ & self.input_bits_

        }*/

    fn new(slowdown: u32) -> GPIO {
        eprintln!("Start building GPIO");

        // Map the GPIO register file. See section 2.1 in the assignment for details
        let map = mmap_bcm_register(GPIO_REGISTER_OFFSET as usize);

        if map.is_none() {
            eprint!("map error");
        }

        // Initialize the GPIO struct with default values
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

                    let mut all_used_bits: u32 = 0;
                    all_used_bits |= GPIO_BIT!(PIN_OE) | GPIO_BIT!(PIN_CLK) | GPIO_BIT!(PIN_LAT) |
                        GPIO_BIT!(PIN_R1) | GPIO_BIT!(PIN_G1) | GPIO_BIT!(PIN_B1) |
                        GPIO_BIT!(PIN_R2) | GPIO_BIT!(PIN_G2) | GPIO_BIT!(PIN_B2);

                    io.row_mask = GPIO_BIT!(PIN_A);
                    let rows_count = ROWS / SUB_PANELS;
                    if rows_count > 2 {
                        io.row_mask |= GPIO_BIT!(PIN_B);
                    }
                    if rows_count > 4 {
                        io.row_mask |= GPIO_BIT!(PIN_C);
                    }
                    if rows_count > 8 {
                        io.row_mask |= GPIO_BIT!(PIN_D);
                    }
                    if rows_count > 16 {
                        io.row_mask |= GPIO_BIT!(PIN_E);
                    }

                    all_used_bits |= io.row_mask;
                    let result = io.init_outputs(all_used_bits);
                    assert!(result == all_used_bits);
                }

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

    // Calculates the pins we must activate to push the address of the specified double_row
    fn get_row_bits(self: &GPIO, double_row: u8) -> u32 {
        //TODO: Implement this yourself.
        let mut row_adress: u32 = 0;
        match double_row & 0x01 as u8 != 0 {
            True => row_adress |= GPIO_BIT!(PIN_A),
            False => row_adress = 0,
        };
        match double_row & 0x02 as u8 != 0 {
            True => row_adress |= GPIO_BIT!(PIN_B),
            False => row_adress = 0,
        };
        match double_row & 0x04 as u8 != 0 {
            True => row_adress |= GPIO_BIT!(PIN_C),
            False => row_adress = 0,
        };
        match double_row & 0x08 as u8 != 0 {
            True => row_adress |= GPIO_BIT!(PIN_D),
            False => row_adress = 0,
        };
        match double_row & 0x10 as u8 != 0 {
            True => row_adress |= GPIO_BIT!(PIN_E),
            False => row_adress = 0,
        };
        row_adress & self.row_mask
    }


}

impl Timer {
    // Reads from the 1Mhz timer register (see Section 2.5 in the assignment)
    unsafe fn read(self: &Timer) -> u32 {
        //TODO: Implement this yourself.
        std::ptr::read_volatile(self.timereg)
    }

    fn new() -> Timer {
        //TODO: Implement this yourself.
        let map = mmap_bcm_register(TIMER_REGISTER_OFFSET as usize);

        let mut timer: Timer = Timer {
            _timemap: None,
            timereg: 0 as *mut u32,
        };

        match &map {
            &Some(ref map) => {
                unsafe {
                    timer.timereg = map.data() as *mut u32;
                    timer.timereg.offset(1);
                }
            }
            &None => {}
        };
        timer
    }

    // High-precision sleep function (see section 2.5 in the assignment)
    // NOTE/WARNING: Since the raspberry pi's timer frequency is only 1Mhz,
    // you cannot reach full nanosecond precision here. You will have to think
    // about how you can approximate the desired precision. Obviously, there is
    // no perfect solution here.
    fn nanosleep(self: &Timer, mut nanos: u32) {
        //TODO: Implement this yourself.
        let mut k_jitter_allowance = 60 * 1000;

        if nanos > k_jitter_allowance {
            let before_microsecs = unsafe { self.read() };
            let nanosec_passed: u64;

            match sleep(std::time::Duration::new(0, nanos - k_jitter_allowance)) {
                Some(time) => {
                    let after_microsec = unsafe { self.read() };
                    if after_microsec > before_microsecs {
                        nanosec_passed = (1000 * (after_microsec - before_microsecs)) as u64;
                    } else {
                        nanosec_passed = 1000 * (TIMER_OVERFLOW_VALUE - before_microsecs + after_microsec) as u64;
                    }
                    if nanosec_passed > nanos as u64 {
                        return;
                    } else {
                        nanos -= nanosec_passed as u32;
                    }
                }
                None => {}
            }
        }

        if nanos < 20 {
            return;
        }

        let mut start_time: u32 = unsafe { self.read() };
        let mut current_time: u32 = start_time;

        while start_time + nanos * 1000 <= current_time {
            current_time = unsafe { self.read() };
        }
        return;
    }
}


// TODO: Implement your frame calculation/updating logic here.
// The Frame should contain the pixels that are currently shown
// on the LED board. In most cases, the Frame will have less pixels
// than the input Image!
impl Frame {
    fn new() -> Frame {
        let mut frame: Frame = Frame {
            pos: 0,
            pixels: vec![vec![Pixel::new(); COLUMNS as usize]; ROWS as usize],
        };
        frame
    }

    fn next_image_frame(self: &mut Frame, image: &Image) {
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

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let interrupt_received = Arc::new(AtomicBool::new(false));

// sanity checks
    if nix::unistd::Uid::current().is_root() == false {
        eprintln!("Must run as root to be able to access /dev/mem\nPrepend \'sudo\' to the command");
        std::process::exit(1);
    }
    else if args.len() < 2 {
        eprintln!("Syntax: {:?} [image]", args[0]);
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    let image = file_reader::read_ppm_file(&path);

    let mut frame = Frame::new();

    frame.next_image_frame(&image);

// TODO: Initialize the GPIO struct and the Timer struct
    let mut gpio = GPIO::new(1);
    println!("GPIO  done");
    let timer = Timer::new();
    println!("Timer done");

    println!("Awaiting exit...");

// This code sets up a CTRL-C handler that writes "true" to the
// interrupt_received bool.
    let int_recv = interrupt_received.clone();
    ctrlc::set_handler(move || {
        int_recv.store(true, Ordering::SeqCst);
    }).unwrap();

    let row_mask = gpio.row_mask;
    let time;
    unsafe {
        time = timer.read();
    }

    while interrupt_received.load(Ordering::SeqCst) == false {
        for row_counter in 0..ROWS / 2 {
            for bitplane_counter in 0..COLOR_DEPTH {
                send_values(&mut gpio, &timer, &frame, row_counter, bitplane_counter);
            }
        }

    };
    println!("Exiting.");
    if interrupt_received.load(Ordering::SeqCst) == true {
        println!("Received CTRL-C");
    } else {
        println!("Timeout reached");
    };

    //TODO
    gpio.set_bits(GPIO_BIT!(PIN_OE));
}

fn send_values(gpio: &mut GPIO, timer: &Timer, frame: &Frame, row: usize, bitplane_counter: usize) {
    let row_mask = gpio.row_mask;
    let color_clock_mask = GPIO_BIT!(PIN_R1) | GPIO_BIT!(PIN_G1) | GPIO_BIT!(PIN_B1) | GPIO_BIT!(PIN_R2) | GPIO_BIT!(PIN_G2) | GPIO_BIT!(PIN_B2) | GPIO_BIT!(PIN_CLK);

    for c in 0..32 {
        gpio.clear_bits(color_clock_mask);
        let pixel_top = frame.pixels[row][c];
        let pixel_bot = frame.pixels[ROWS / 2 + row][c];
        let plane_bits = get_plane_bits(pixel_top, pixel_bot, bitplane_counter);

        gpio.write_masked_bits(plane_bits, color_clock_mask);
        gpio.set_bits(GPIO_BIT!(PIN_CLK));
    };
    gpio.clear_bits(GPIO_BIT!(PIN_R1) | GPIO_BIT!(PIN_G1) | GPIO_BIT!(PIN_B1) | GPIO_BIT!(PIN_R2) | GPIO_BIT!(PIN_G2) | GPIO_BIT!(PIN_B2) | GPIO_BIT!(PIN_CLK));

    gpio.write_masked_bits(get_row_bits(row), row_mask);

    gpio.set_bits(GPIO_BIT!(PIN_LAT));
    gpio.clear_bits(GPIO_BIT!(PIN_LAT));
    gpio.clear_bits(GPIO_BIT!(PIN_OE));
    timer.nanosleep(gpio.bitplane_timings[bitplane_counter] as u32);
    gpio.set_bits(GPIO_BIT!(PIN_OE));
}

//TODO: Not sure if needed, in C reference code, not in RUST skeleton code.
pub fn get_plane_bits(top: Pixel, bot: Pixel, plane: usize) -> u32 {
    let mut out: u32 = 0;
    match top.r & (1 << plane) != 0 {
        True => out |= GPIO_BIT!(PIN_R1),
        False => out = 0,
    };
    match bot.r & (1 << plane) != 0 {
        True => out |= GPIO_BIT!(PIN_R2),
        False => out = 0,
    };
    match top.g & (1 << plane) != 0 {
        True => out |= GPIO_BIT!(PIN_G1),
        False => out = 0,
    };
    match bot.g & (1 << plane) != 0 {
        True => out |= GPIO_BIT!(PIN_G2),
        False => out = 0,
    };
    match top.b & (1 << plane) != 0 {
        True => out |= GPIO_BIT!(PIN_B1),
        False => out = 0,
    };
    match bot.b & (1 << plane) != 0 {
        True => out |= GPIO_BIT!(PIN_B2),
        False => out = 0,
    };
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