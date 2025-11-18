// Welcome to
// __________         __    __  .__                               __
// \______   \_____ _/  |__/  |_|  |   ____   ______ ____ _____  |  | __ ____
//  |    |  _/\__  \\   __\   __\  | _/ __ \ /  ___//    \\__  \ |  |/ // __ \
//  |    |   \ / __ \|  |  |  | |  |_\  ___/ \___ \|   |  \/ __ \|    <\  ___/
//  |________/(______/__|  |__| |____/\_____>______>___|__(______/__|__\\_____>
//
// This file can be a nice home for your Battlesnake logic and helper functions.
//
// To get you started we've included code to prevent your Battlesnake from moving backwards.
// For more info see docs.battlesnake.com

use log::info;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

use crate::{Battlesnake, Board, Coord, Game};

impl Coord {
    fn neighbors(&self) -> [Coord; 4] {
        [
            Coord {
                x: self.x,
                y: self.y + 1,
            },
            Coord {
                x: self.x,
                y: self.y - 1,
            },
            Coord {
                x: self.x + 1,
                y: self.y,
            },
            Coord {
                x: self.x - 1,
                y: self.y,
            },
        ]
    }
}
// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "MylesJPritchett",
        "color": "#FF5F1F",
        "head": "default",
        "tail": "default",
    });
}

// start is called when your Battlesnake begins a game
pub fn start(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME START");
}

// end is called when your Battlesnake finishes a game
pub fn end(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME OVER");
}

// move is called on every turn and returns your next move
// Valid moves are "up", "down", "left", or "right"
// See https://docs.battlesnake.com/api/example-move for available data
pub fn get_move(_game: &Game, turn: &i32, board: &Board, you: &Battlesnake) -> Value {
    println!("TURN {}", turn);
    let mut is_move_safe: HashMap<_, _> = vec![
        ("up", true),
        ("down", true),
        ("left", true),
        ("right", true),
    ]
    .into_iter()
    .collect();

    // We've included code to prevent your Battlesnake from moving backwards
    let my_head = &you.body[0]; // Coordinates of your head
    let _my_neck = &you.body[1]; // Coordinates of your "neck"

    // Step 1 - Prevent your Battlesnake from moving out of bounds
    let board_width = &board.width;
    let board_height = &board.height;
    if my_head.x == 0 {
        // Head is on left edge, don't move left
        is_move_safe.insert("left", false);
    } else if my_head.x == board_width - 1 {
        // Head is on right edge, don't move right
        is_move_safe.insert("right", false);
    }

    if my_head.y == 0 {
        // Head is on bottom edge, don't move down
        is_move_safe.insert("down", false);
    } else if my_head.y == board_height - 1 {
        // Head is on top edge, don't move up
        is_move_safe.insert("up", false);
    }

    // Step 2 - Prevent your Battlesnake from colliding with itself
    //
    let lookers = HashMap::from([
        (
            Coord {
                x: my_head.x,
                y: my_head.y + 1,
            },
            "up",
        ),
        (
            Coord {
                x: my_head.x,
                y: my_head.y - 1,
            },
            "down",
        ),
        (
            Coord {
                x: my_head.x - 1,
                y: my_head.y,
            },
            "left",
        ),
        (
            Coord {
                x: my_head.x + 1,
                y: my_head.y,
            },
            "right",
        ),
    ]);

    let my_body = &you.body;

    for body_part in my_body.iter() {
        if lookers.contains_key(body_part) {
            let direction = lookers.get(body_part).unwrap();
            println!("Body part is in {:?}", direction);
            is_move_safe.insert(direction, false);
        }
    }

    // Step 3 - Prevent your Battlesnake from colliding with other Battlesnakes
    let opponents = &board.snakes;
    for oppponent in opponents {
        let opponent_body = &oppponent.body;
        for body_part in opponent_body {
            if lookers.contains_key(body_part) {
                let direction = lookers.get(body_part).unwrap();
                println!("Body of opponent part is in {:?}", direction);
                is_move_safe.insert(direction, false);
            }
        }
    }
    // Are there any safe moves left?
    let safe_moves = is_move_safe
        .into_iter()
        .filter(|&(_, v)| v)
        .map(|(k, _)| k)
        .collect::<Vec<_>>();

    // Choose a random move from the safe ones

    // TODO: Step 4 - Move towards food instead of random, to regain health and survive longer
    /*
    impl Coord {
        fn successors(&self) -> Vec<Coord> {
            let &Coord(x, y) = self;
            vec![
                Coord(x, y + 1),
                Coord(x, y - 1),
                Coord(x - 1, y),
                Coord(x + 1, y),
            ]
        }
    }
    let food = &board.food;
    let result = bfs(&my_head, |p| p.successors(), |p| *p == food&food, &board);
    */

    let food = &board.food;
    let food_distances: HashMap<&Coord, i32> = food
        .iter()
        .map(|food| {
            (
                food,
                (food.x - my_head.x).abs() + (food.y - my_head.y).abs(),
            )
        })
        .collect();

    let closest_food = food_distances
        .iter()
        .min_by_key(|&(_, v)| v)
        .map(|(k, _)| k)
        .unwrap();
    let mut desired_move = "left";

    match closest_food {
        Coord { x, y } => {
            if *x < my_head.x {
                // Food is left of head, move left
                desired_move = "left";
            } else if *x > my_head.x {
                // Food is right of head, move right
                desired_move = "right";
            } else if *y < my_head.y {
                // Food is below head, move down
                desired_move = "down";
            } else if *y > my_head.y {
                // Food is above head, move up
                desired_move = "up";
            }
        }
    }
    println!("Safe moves: {:?}", safe_moves);

    if !safe_moves.is_empty() {
        let mut max_count = 0;
        let mut flood_fill_direction = HashMap::new();
        for &safe_move in &safe_moves {
            let count = match safe_move {
                "up" => flood_fill_count(
                    &board,
                    &Coord {
                        x: my_head.x,
                        y: my_head.y + 1,
                    },
                ),
                "down" => flood_fill_count(
                    &board,
                    &Coord {
                        x: my_head.x,
                        y: my_head.y - 1,
                    },
                ),
                "left" => flood_fill_count(
                    &board,
                    &Coord {
                        x: my_head.x - 1,
                        y: my_head.y,
                    },
                ),
                "right" => flood_fill_count(
                    &board,
                    &Coord {
                        x: my_head.x + 1,
                        y: my_head.y,
                    },
                ),
                _ => {
                    println!("Unknown direction");
                    Ok(0) // Default to 0 in case of an unknown direction
                }
            };

            if let Ok(count) = count {
                flood_fill_direction.insert(safe_move, count);
                max_count = max_count.max(count);
            } else {
                // Handle the error case if flood_fill_count returns an error
                println!("Error calculating flood fill count");
            }
        }
        println!("Flood fill counts: {:?}", flood_fill_direction);
        // return a list of the moves with highest flood fill count
        let best_moves: Vec<&str> = flood_fill_direction
            .iter()
            .filter_map(|(&dir, &count)| if count == max_count { Some(dir) } else { None })
            .collect();
        if best_moves.contains(&desired_move) {
            println!("Desired move is safe");
            json!({ "move": desired_move })
        } else {
            println!("Desired move is not safe");
            json!({ "move": best_moves[0] })
        }
    } else {
        println!("No safe moves");
        json!({ "move": "up" })
    }
}

fn flood_fill_count(board: &Board, start: &Coord) -> Result<i32, &'static str> {
    let mut visited = HashSet::new();
    visited.insert(start.clone());

    let mut frontier = vec![start.clone()];

    while !frontier.is_empty() {
        let mut new_frontier = vec![];

        for coord in frontier {
            let neighbors = coord.neighbors();

            for neighbor in neighbors {
                if !visited.contains(&neighbor) && board.is_empty(&neighbor) {
                    new_frontier.push(neighbor.clone());
                    visited.insert(neighbor);
                }
            }
        }

        frontier = new_frontier;
    }

    Ok(visited.len() as i32)
}
