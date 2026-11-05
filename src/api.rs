//! Battlesnake API types, see https://docs.battlesnake.com/api

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub struct Game {
    pub id: String,
    pub ruleset: HashMap<String, Value>,
    pub timeout: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Board {
    pub height: i32,
    pub width: i32,
    pub food: Vec<Coord>,
    pub snakes: Vec<Battlesnake>,
    pub hazards: Vec<Coord>,
}

impl Board {
    pub fn contains(&self, coord: &Coord) -> bool {
        (0..self.width).contains(&coord.x) && (0..self.height).contains(&coord.y)
    }

    pub fn is_snake_body(&self, coord: &Coord) -> bool {
        self.snakes.iter().any(|snake| snake.body.contains(coord))
    }

    pub fn is_hazard(&self, coord: &Coord) -> bool {
        self.hazards.contains(coord)
    }

    /// A cell is free when it is on the board and holds no snake body or hazard.
    pub fn is_free(&self, coord: &Coord) -> bool {
        self.contains(coord) && !self.is_snake_body(coord) && !self.is_hazard(coord)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Battlesnake {
    pub id: String,
    pub name: String,
    pub health: i32,
    pub body: Vec<Coord>,
    pub head: Coord,
    pub length: i32,
    pub latency: String,
    pub shout: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub fn manhattan_distance(&self, other: &Coord) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GameState {
    pub game: Game,
    pub turn: i32,
    pub board: Board,
    pub you: Battlesnake,
}
