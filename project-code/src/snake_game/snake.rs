use std::collections::LinkedList;
use super::super::utils::frame::Frame;
use super::super::utils::pixel::Pixel;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::UP => Direction::DOWN,
            Direction::DOWN => Direction::UP,
            Direction::LEFT => Direction::RIGHT,
            Direction::RIGHT => Direction::LEFT,
        }
    }
}

#[derive(Debug, Clone)]
struct Block {
    x: i32,
    y: i32,
}

pub struct Snake {
    SNAKE_BLOCK: Pixel,
    moving_direction: Direction,
    body: LinkedList<Block>,
    last_removed_block: Option<Block>,
}

impl Snake {
    pub fn new(init_x: i32, init_y: i32) -> Snake {
        let mut body: LinkedList<Block> = LinkedList::new();
        body.push_back(Block {
            x: init_x + 2,
            y: init_y,
        });
        body.push_back(Block {
            x: init_x + 1,
            y: init_y,
        });
        body.push_back(Block {
            x: init_x,
            y: init_y,
        });

        Snake {
            SNAKE_BLOCK: Pixel::new_colored_pixel(0, 255, 0),
            moving_direction: Direction::RIGHT,
            body: body,
            last_removed_block: None,
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        for block in &self.body {
            frame.pixels[block.x as usize][block.y as usize] = self.SNAKE_BLOCK;
        }
    }

    pub fn move_forward(&mut self, dir: Option<Direction>) {
        // Change moving direction
        match dir {
            Some(d) => self.moving_direction = d,
            None => {}
        }

        // Retrieve the position of the head block
        let (last_x, last_y): (i32, i32) = self.head_position();

        // The snake moves
        let new_block = match self.moving_direction {
            Direction::UP => Block {
                x: last_x - 1,
                y: last_y,
            },
            Direction::DOWN => Block {
                x: last_x + 1,
                y: last_y,
            },
            Direction::LEFT => Block {
                x: last_x,
                y: last_y - 1,
            },
            Direction::RIGHT => Block {
                x: last_x,
                y: last_y + 1,
            }
        };
        self.body.push_front(new_block);
        let removed_blk = self.body.pop_back().unwrap();
        self.last_removed_block = Some(removed_blk);
    }

    pub fn head_position(&self) -> (i32, i32) {
        let head_block = self.body.front().unwrap();
        (head_block.x, head_block.y)
    }

    pub fn head_direction(&self) -> Direction {
        self.moving_direction
    }

    pub fn next_head_position(&self, dir: Option<Direction>) -> (i32, i32) {
        // Retrieve the position of the head block
        let (head_x, head_y): (i32, i32) = self.head_position();

        println!("head position({},{})", head_x, head_y);

        // Get moving direction
        let mut moving_dir = self.moving_direction;
        match dir {
            Some(d) => moving_dir = d,
            None => {}
        }

        println!("Move direction {:?}", moving_dir);

        // The snake moves
        match moving_dir {
            Direction::UP => (head_x - 1, head_y),
            Direction::DOWN => (head_x + 1, head_y),
            Direction::LEFT => (head_x, head_y - 1),
            Direction::RIGHT => (head_x, head_y + 1)
        }
    }

    pub fn restore_last_removed(&mut self) {
        let blk = self.last_removed_block.clone().unwrap();
        self.body.push_back(blk);
    }

    pub fn is_overlap_except_tail(&self, x: i32, y: i32) -> bool {
        let mut checked = 0;
        for block in &self.body {
            if x == block.x && y == block.y {
                return true;
            }

            // For excluding the tail
            checked += 1;
            if checked == self.body.len() - 1 {
                break;
            }
        }
        return false;
    }

    pub fn get_body_blocks(&self){
        self.body
    }
}