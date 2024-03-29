// Skeleton code for your Rust projects
// I added several comments and annotations to this file.
// _Please_ read them carefully. They are very important.
// The most important comments are all annotated with "NOTE/WARNING:"

// I will grade your code quality primarily on how "idiomatic" your Rust 
// code is, and how well you implemented the "safe unsafety" guidelines.

extern crate libc;
extern crate time;
extern crate ctrlc;
#[macro_use]
extern crate simple_error;
extern crate shuteye;
extern crate mmap;
extern crate nix;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{fs::OpenOptions, os::unix::fs::OpenOptionsExt};
use std::error::Error;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use std::time::Duration;
use shuteye::sleep;
use mmap::{MemoryMap, MapOption};

#[derive(Copy, Clone)]
struct Pixel {
    r: u16,
    g: u16,
    b: u16,
}

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
    row_mask: u32,
    bitplane_timings: [u32; COLOR_DEPTH],
}

// This is a representation of the "raw" image
struct Image {
    width: usize,
    height: usize,
    pixels: Vec<Vec<Pixel>>,
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
const REGISTER_BLOCK_SIZE: u64 = 4096;
const COLOR_DEPTH: usize = 8;

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

// ============================================================================
// mmap_bcm_register - convenience function used to map the GPIO register block
// ============================================================================

fn mmap_bcm_register(register_offset: usize) -> Option<MemoryMap> {
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

    let result = MemoryMap::new(REGISTER_BLOCK_SIZE as usize, mmap_options).unwrap();

    return match result.data().is_null() {
        true => {
            eprintln!("mmap error: {}", std::io::Error::last_os_error());
            eprintln!("Pi3: MMapping from base 0x{:X}, offset 0x{:X}", BCM2709_PERI_BASE, register_offset);
            None
        }
        false => Some(result)
    };

    // NOTE/WARNING: When a MemoryMap struct is dropped, the mapped 
    // memory region is automatically unmapped!
}

//
// NOTE/WARNING: In many cases, particularly those where you need to set or clear 
// multiple bits at once, it is convenient to store multiple pin numbers in one bit 
// mask value. If you want to simultaneously set PIN_A and PIN_C to high, for example, 
// you should probably create a bit mask with the positions of PIN_A and PIN_C set to 1, 
// and all other positions set to 0. You can do this using the GPIO_BIT! macro.
//
// In this example, you would do something like:
//     let pin_mask = GPIO_BIT!(PIN_A) | GPIO_BIT!(PIN_C);
//     io.set_bits(pin_mask);
//
impl GPIO {
    // configures pin number @pin_num as an output pin by writing to the
    // appropriate Function Select register (see section 2.1).
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
        // TODO: Implement this yourself. Note: this function expects
        // a bitmask as the @outputs argument
        outputs &= VALID_BITS as u32;
        outputs &= !(self.output_bits_ | self.input_bits_);

        for b in 0..=27 {
            if GPIO_BIT!(b) & outputs != 0 {
                self.configure_output_pin(b as u64);
            }
        }
        self.output_bits_ |= outputs;

        outputs
    }


    fn set_bits(self: &mut GPIO, value: u32) {
        // TODO: Implement this yourself. Remember to take the slowdown_ value into account!
        // This function expects a bitmask as the @value argument
        self.gpio_set_bits_ = value as *mut u32;
        for i in 0..self.slowdown_ {
            self.gpio_set_bits_ = value as *mut u32;
        }
    }

    fn clear_bits(self: &mut GPIO, value: u32) {
        // TODO: Implement this yourself. Remember to take the slowdown_ value into account!
        // This function expects a bitmask as the @value argument
    }

    // Write all the bits of @value that also appear in @mask. Leave the rest untouched.
    // @value and @mask are bitmasks
    fn write_masked_bits(
        self: &mut GPIO,
        value: u32,
        mask: u32,
    ) {
        // TODO: Implement this yourself.
    }

    fn new(slowdown: u32) -> GPIO {

        // Map the GPIO register file. See section 2.1 in the assignment for details
        let map = mmap_bcm_register(GPIO_REGISTER_OFFSET as usize);

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

        /* match &map {
             Some(m) => {
                 unsafe {
                     io.gpio_port_ = m.data() as *mut u32;
                     // TODO: Calculate the correct values of the other raw pointers here.
                     // You should use the offset() method on the gpio_port_ pointer.
                     // Keep in mind that Rust raw pointer arithmetic works exactly like
                     // C pointer arithmetic. See the course slides for details
                 }

                 // TODO: Implement this yourself.
             },
             None => {}
         }

         io.gpio_map_ = map;*/
        io
    }

    /*   // Calculates the pins we must activate to push the address of the specified double_row
       fn get_row_bits(self: &GPIO, double_row: u8) -> u32 {
           // TODO: Implement this yourself.
       }*/
}
/*
impl Timer {
    // Reads from the 1Mhz timer register (see Section 2.5 in the assignment)
    unsafe fn read(self: &Timer) -> u32 {
        // TODO: Implement this yourself.
    }

    fn new() -> Timer {
        // TODO: Implement this yourself.
    }

    // High-precision sleep function (see section 2.5 in the assignment)
    // NOTE/WARNING: Since the raspberry pi's timer frequency is only 1Mhz, 
    // you cannot reach full nanosecond precision here. You will have to think
    // about how you can approximate the desired precision. Obviously, there is
    // no perfect solution here.
    fn nanosleep(self: &Timer, mut nanos: u32) {
        // TODO: Implement this yourself.
    }
}

// TODO: Implement your frame calculation/updating logic here.
// The Frame should contain the pixels that are currently shown
// on the LED board. In most cases, the Frame will have less pixels
// than the input Image!
impl Frame {

}

// TODO: Add your PPM parser here
// NOTE/WARNING: Please make sure that your implementation can handle comments in the PPM file
// You do not need to add support for any formats other than P6
// You may assume that the max_color value is always 255, but you should add sanity checks
// to safely reject files with other max_color values
impl Image {

}*/

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let interrupt_received = Arc::new(AtomicBool::new(false));

    // sanity checks
    if nix::unistd::Uid::current().is_root() == false {
        eprintln!("Must run as root to be able to access /dev/mem\nPrepend \'sudo\' to the command");
        std::process::exit(1);
    }

    println!("Starting Program");


    println!("Waiting for termination...");

    // TODO: Read the PPM file here. You can find its name in args[1]
    // TODO: Initialize the GPIO struct and the Timer struct

    // This code sets up a CTRL-C handler that writes "true" to the 
    // interrupt_received bool.
    let int_recv = interrupt_received.clone();
    ctrlc::set_handler(move || {
        int_recv.store(true, Ordering::SeqCst);
    }).unwrap();

    while interrupt_received.load(Ordering::SeqCst) == false {
        // TODO: Implement your rendering loop here
    }
    println!("Exiting.");
    if interrupt_received.load(Ordering::SeqCst) == true {
        println!("Received CTRL-C");
    } else {
        println!("Timeout reached");
    }

    // TODO: You may want to reset the board here (i.e., disable all LEDs)
}