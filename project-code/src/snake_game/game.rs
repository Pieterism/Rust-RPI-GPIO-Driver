extern crate rand;

use time;
use time::Timespec;

use super::snake::{Direction, Snake};
use super::super::utils::frame::Frame;
use super::super::utils::gpio_driver::{COLUMNS, ROWS};
use super::super::utils::pixel::Pixel;

use self::rand::Rng;
use self::rand::thread_rng;

const MOVING_PERIOD: i64 = 200;
const RESTART_TIME: i64 = 3000;

pub struct Game {
    snake: Snake,
    food_exist: bool,
    food_x: i32,
    food_y: i32,
    FOOD_BLOCK: Pixel,
    width: i32,
    height: i32,
    is_game_over: bool,
}

impl Game {
    pub fn new() -> Game {
        Game {
            snake: Snake::new(2, 2),
            food_exist: true,
            food_x: 5,
            food_y: 3,
            FOOD_BLOCK: Pixel::new_colored_pixel(255, 0, 0),
            width: COLUMNS as i32,
            height: ROWS as i32,
            is_game_over: false,
        }
    }

    pub fn key_pressed(&mut self, dir: Option<Direction>) {
        if dir.is_none() {
            return;
        }

        if dir.unwrap() == self.snake.head_direction().opposite() {
            return;
        }

        self.update_snake(dir);
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.clear_frame();
        self.snake.draw(frame);

        if self.food_exist {
            frame.pixels[self.food_x as usize][self.food_y as usize] = self.FOOD_BLOCK;
        }

        frame.draw_border();
        if self.is_game_over {frame.draw_game_over()}
    }

    pub fn update(&mut self, prev_time_frame: &mut Timespec) -> bool {
        if self.is_game_over {
            if (time::get_time() - *prev_time_frame) >= time::Duration::milliseconds(RESTART_TIME) {
                self.restart();
            }
            return false;
        }

        if !self.food_exist {
            self.add_food();
        }

        if (time::get_time() - *prev_time_frame) >= time::Duration::milliseconds(MOVING_PERIOD) {
            self.update_snake(None);
            return true;
        };
        return false;
    }

    pub fn is_game_over(&mut self) -> bool {
        self.is_game_over
    }


    //TODO check if eating
/*    fn check_eating(&mut self) {
        let (head_x, head_y): (i32, i32) = self.snake.head_position();
        for bl in self.snake.get_body_blocks(){
            if self.food_exist && self.food_x == bl.x && self.food_y == bl.y {
                self.food_exist = false;
                self.snake.restore_last_removed();
            }
        }

    }
*/
    fn check_if_the_snake_alive(&self, dir: Option<Direction>) -> bool {
        let (next_x, next_y) = self.snake.next_head_position(dir);

        if self.snake.is_overlap_except_tail(next_x, next_y) {
            return false;
        }

        next_x > 0 && next_y > 0 && next_x < self.height - 1 && next_y < self.width - 1
    }

    fn add_food(&mut self) {
        let mut rng = thread_rng();
        let mut new_x = rng.gen_range(1, self.width - 1);
        let mut new_y = rng.gen_range(1, self.width - 1);

        while self.snake.is_overlap_except_tail(new_x, new_y) {
            new_x = rng.gen_range(1, self.width - 1);
            new_y = rng.gen_range(1, self.width - 1);
        }

        self.food_x = new_x;
        self.food_y = new_y;
        self.food_exist = true;
    }

    fn update_snake(&mut self, dir: Option<Direction>) {
        if self.check_if_the_snake_alive(dir) {
            eprintln!("Moving forward");
            self.snake.move_forward(dir);
            //self.check_eating();
        }
        else {
            self.is_game_over = true;
        }
    }

    fn restart(&mut self) {
        self.snake = Snake::new(2, 2);
        self.food_exist = true;
        self.food_x = 5;
        self.food_y = 3;
        self.is_game_over = false;
    }
}