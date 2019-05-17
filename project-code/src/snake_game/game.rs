extern crate rand;

use super::snake::{Snake, Direction};
use self::rand::thread_rng;
use self::rand::Rng;
use super::super::utils::gpio_driver::{COLUMNS, ROWS};
use super::super::utils::frame::Frame;
use super::super::utils::pixel::Pixel;


const MOVING_PERIOD: f64 = 0.2;
// in second
const RESTART_TIME: f64 = 3.0; // in second

pub struct Game {
    snake: Snake,

    // Food
    food_exist: bool,
    food_x: i32,
    food_y: i32,
    FOOD_BLOCK: Pixel,
    // Game Space
    width: i32,
    height: i32,

    // Game state
    is_game_over: bool,
    // When the game is running, it represents the waiting time from the previous moving
    // When the game is over, it represents the waiting time from the end of the game
    waiting_time: f64,
}

impl Game {
    pub fn new() -> Game {
        Game {
            snake: Snake::new(2, 2),
            waiting_time: 0.0,
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
        println!("Key pressed: {:?}",dir);

        if dir.is_none() {
            return;
        }
        if dir.unwrap() == self.snake.head_direction().opposite() {
            println!("opposite direction");
            return;
        }

        // Check if the snake hits the border
        self.update_snake(dir);
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.clear_frame();
        //Draw the snake

        self.snake.draw(frame);

        // Draw the food
        if self.food_exist {
            frame.pixels[self.food_x as usize][self.food_y as usize] = self.FOOD_BLOCK;
        }

        //Draw the border
        frame.draw_border();

        //Draw game over
        if self.is_game_over { frame.draw_game_over() }
    }

    pub fn update(&mut self, delta_time: f64) {
        self.waiting_time += delta_time;

        // If the game is over
        if self.is_game_over {
            if self.waiting_time > RESTART_TIME {
                self.restart();
            }
            return;
        }

        // Check if the food still exists
        if !self.food_exist {
            self.add_food();
        }

        // Move the snake
        if self.waiting_time > MOVING_PERIOD {
            self.update_snake(None);
        }
    }

    pub fn is_game_over(&mut self) -> bool {
        self.is_game_over
    }

    fn check_eating(&mut self) {
        let (head_x, head_y): (i32, i32) = self.snake.head_position();
        if self.food_exist && self.food_x == head_x && self.food_y == head_y {
            self.food_exist = false;
            self.snake.restore_last_removed();
        }
    }

    fn check_if_the_snake_alive(&self, dir: Option<Direction>) -> bool {
        let (next_x, next_y) = self.snake.next_head_position(dir);

        println!("check snake alive next head position ({},{})", next_x, next_y);
        // Check if the snake hits itself
        if self.snake.is_overlap_except_tail(next_x, next_y) {
            return false;
        }

        // Check if the snake overlaps with the border
        next_x > 0 && next_y > 0 && next_x < self.width - 1 && next_y < self.height - 1
    }

    fn add_food(&mut self) {
        let mut rng = thread_rng();

        // Decide the position of the new food
        let mut new_x = rng.gen_range(1, self.width - 1);
        let mut new_y = rng.gen_range(1, self.width - 1);
        while self.snake.is_overlap_except_tail(new_x, new_y) {
            new_x = rng.gen_range(1, self.width - 1);
            new_y = rng.gen_range(1, self.width - 1);
        }

        // Add the new food
        self.food_x = new_x;
        self.food_y = new_y;
        self.food_exist = true;
    }

    fn update_snake(&mut self, dir: Option<Direction>) {
        println!("Update snake: {:?}", dir);
        if self.check_if_the_snake_alive(dir) {
            eprintln!("Moving forward");
            self.snake.move_forward(dir);
            self.check_eating();
        } else {
            self.is_game_over = true;
        }
        self.waiting_time = 0.0;
    }

    fn restart(&mut self) {
        self.snake = Snake::new(2, 2);
        self.waiting_time = 0.0;
        self.food_exist = true;
        self.food_x = 5;
        self.food_y = 3;
        self.is_game_over = false;
    }
}