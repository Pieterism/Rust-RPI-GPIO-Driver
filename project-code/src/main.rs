extern crate ctrlc;
extern crate libc;
extern crate mmap;
extern crate nix;
extern crate shuteye;
#[macro_use]
extern crate simple_error;
extern crate termion;
extern crate time;

use std::io::stdout;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use termion::input::TermRead;
use termion::raw::IntoRawMode;

use snake_game::game::*;
use snake_game::snake::*;
use utils::file_reader;
use utils::frame::Frame;
use utils::gpio_driver::GPIO;
use utils::time::Timer;

mod utils;
mod snake_game;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    sanity_check(&args);

    let interrupt_received = Arc::new(AtomicBool::new(false));
    let mut gpio = GPIO::new(1);
    let timer = Timer::new();
    let mut frame = Frame::new();

    if &args[1] == "snake" {
        println!("Starting Snake");
        let mut game = Game::new();
        let int_recv = interrupt_received.clone();
        let (tx, rx) = mpsc::channel::<Option<Direction>>();
        let mut prev_frame_time = time::get_time();

        thread::spawn(move || loop {
            let buffer: Option < Direction >;
            buffer = wait_for_key_press();
            tx.send(buffer).unwrap();
            println!("send!");
        });

        ctrlc::set_handler(move || {
            int_recv.store(true, Ordering::SeqCst);
        }).unwrap();

        game.draw(&mut frame);
        loop{
            if !game.is_game_over(){
                let option = match rx.try_recv() {
                    Ok(dir) => dir,
                    Err(_err) => None
                };

                if option.is_some() {
                    println!("option: {:?}", option);
                }

                game.key_pressed(option);
                if game.update(&mut prev_frame_time) {
                    prev_frame_time = time::get_time();
                    game.draw(&mut frame);
                }
            }
            gpio.render_frame(&mut frame, &timer);
        }
    }
    //RENDER IMAGE
    else {
        println!("Rendering Image");
        let path = Path::new(&args[1]);
        let image = file_reader::read_ppm_file(&path);
        let int_recv = interrupt_received.clone();

        ctrlc::set_handler(move || {
            int_recv.store(true, Ordering::SeqCst);
        }).unwrap();

        gpio.render_image_frame(interrupt_received, &image, &mut frame, &timer, true);
    }
}

fn sanity_check(args: &Vec<String>) {
    if nix::unistd::Uid::current().is_root() == false {
        eprintln!("Must run as root to be able to access /dev/mem\nPrepend \'sudo\' to the command");
        std::process::exit(1);
    } else if args.len() < 2 {
        eprintln!("Syntax: {:?} [image]", args[0]);
        std::process::exit(1);
    }
}

fn wait_for_key_press() -> Option<Direction> {
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut stdin = termion::async_stdin().keys();
    let dir: Option<Direction>;

    println!("init loop");

    loop{
        let input = stdin.next();
        if let Some(Ok(key)) = input {
            match key {
                termion::event::Key::Up => {
                    dir = Some(Direction::UP);
                    break;
                }
                termion::event::Key::Down => {
                    dir = Some(Direction::DOWN);
                    break;
                }
                termion::event::Key::Right => {
                    dir = Some(Direction::RIGHT);
                    break;
                }
                termion::event::Key::Left => {
                    dir = Some(Direction::LEFT);
                    break;
                }
                termion::event::Key::Esc => {
                    println!("Esc pressed");
                    exit(0);
                }
                _ => {
                }
            };
            stdout.flush().unwrap();
        };
        thread::sleep(Duration::from_millis(10 ));
    };
    dir
}

