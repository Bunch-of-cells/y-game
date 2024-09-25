use log::warn;

use crate::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, Default)]
pub struct Game {
    red: u128,
    blue: u128,
    last_move: u32,
    red_to_move: bool,
    state: GameState,
}

#[derive(Debug, Clone, PartialEq, Default, Copy)]
pub enum GameState {
    RedWin,
    RedLose,
    #[default]
    OnGoing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameError {
    InvalidMove,
    GameEnded,
}

impl Game {
    pub fn new(boards: (u128, u128), red_to_move: bool) -> Game {
        Game {
            red: boards.0,
            blue: boards.1,
            red_to_move,
            last_move: 93,
            ..Default::default()
        }
    }

    pub fn boards(&self) -> (u128, u128) {
        (self.red, self.blue)
    }

    pub fn reset(&mut self, red: u128, blue: u128, red_to_move: bool) {
        self.red = red;
        self.blue = blue;
        self.red_to_move = red_to_move;
        self.last_move = 93;
        self.state = GameState::OnGoing;
    }

    pub fn state(&self) -> GameState {
        self.state
    }

    pub(crate) fn play_inner(&mut self, node: u32) {
        let mask = 1 << node;
        if self.red_to_move {
            self.red |= mask;
        } else {
            self.blue |= mask;
        }
        self.last_move = node;
        self.red_to_move = !self.red_to_move;
    }

    pub fn play(&mut self, node: u32) -> Result<(), GameError> {
        if self.state != GameState::OnGoing {
            return Err(GameError::GameEnded);
        }
        if node > 92 {
            return Err(GameError::InvalidMove);
        }
        let board = self.red | self.blue;
        let mask = 1 << node;
        if board & mask != 0 {
            warn!("Not playable move");
            return Err(GameError::InvalidMove);
        }
        if self.red_to_move {
            self.red |= mask;
        } else {
            self.blue |= mask;
        }
        self.last_move = node;
        self.update_winner();
        self.red_to_move = !self.red_to_move;
        Ok(())
    }

    pub fn get(&self, node: u32) -> u8 {
        let mask = 1 << node;
        if self.red & mask != 0 {
            1
        } else if self.blue & mask != 0 {
            2
        } else {
            0
        }
    }

    pub fn player(&self) -> u8 {
        if self.red_to_move {
            1
        } else {
            2
        }
    }

    pub fn last_move(&self) -> u32 {
        self.last_move
    }

    pub fn update_winner(&mut self) {
        if self.last_move == 93 {
            return;
        }
        let mut q = VecDeque::new();
        let mut visited = 0u128;

        let mut touched_sides = [false; 3];

        visited |= 1 << self.last_move;
        q.push_back(self.last_move);
        while !q.is_empty() {
            let current = q.pop_front().unwrap();
            for (i, side) in SIDES.iter().enumerate() {
                if side & (1 << current) != 0 {
                    touched_sides[i] = true;
                }
            }

            if touched_sides.iter().all(|&t| t) {
                if self.red_to_move {
                    self.state = GameState::RedWin;
                } else {
                    self.state = GameState::RedLose;
                }
            }

            for adj in NEIGHBOURS[current as usize] {
                if adj == 0 {
                    continue;
                }
                let adj = adj - 1;
                let mask = 1 << adj;
                if visited & mask == 0 && {
                    (if self.red_to_move {
                        self.red
                    } else {
                        self.blue
                    }) & mask
                        != 0
                } {
                    visited |= 1 << adj;
                    q.push_back(adj);
                }
            }
        }
    }
}
