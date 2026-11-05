use log::{debug, info};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashSet;

use crate::api::{Board, Coord, GameState};

/// One of the four directions the head can go this turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Move {
    Up,
    Down,
    Left,
    Right,
}

impl Move {
    /// Every move, in the fixed order used whenever moves tie.
    pub const ALL: [Move; 4] = [Move::Up, Move::Down, Move::Left, Move::Right];

    pub fn step(self, from: Coord) -> Coord {
        match self {
            Move::Up => Coord {
                x: from.x,
                y: from.y + 1,
            },
            Move::Down => Coord {
                x: from.x,
                y: from.y - 1,
            },
            Move::Left => Coord {
                x: from.x - 1,
                y: from.y,
            },
            Move::Right => Coord {
                x: from.x + 1,
                y: from.y,
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MoveResponse {
    pub r#move: Move,
}

pub fn info() -> Value {
    info!("INFO");

    json!({
        "apiversion": "1",
        "author": "MylesJPritchett",
        "color": "#FF5F1F",
        "head": "default",
        "tail": "default",
    })
}

pub fn start(_state: &GameState) {
    info!("GAME START");
}

pub fn end(_state: &GameState) {
    info!("GAME OVER");
}

pub fn get_move(state: &GameState) -> MoveResponse {
    info!("TURN {}", state.turn);
    let chosen = choose_move(state);
    info!("Chosen move: {chosen:?}");
    MoveResponse { r#move: chosen }
}

/// The move decision for a game state: space first, food second.
pub fn choose_move(state: &GameState) -> Move {
    let board = &state.board;
    let head = state.you.body[0];
    let food = nearest_food(board, head);

    let safe = safe_moves(state);
    debug!("Safe moves: {safe:?}");
    if safe.is_empty() {
        info!("No safe moves");
        return Move::Up;
    }

    let best = best_moves(board, head, &safe);
    debug!("Best moves: {best:?}");
    choose_among_best(&best, head, food)
}

/// Moves that don't leave the board or land on any snake's body.
fn safe_moves(state: &GameState) -> Vec<Move> {
    let board = &state.board;
    let head = state.you.body[0];
    Move::ALL
        .iter()
        .copied()
        .filter(|mv| {
            let target = mv.step(head);
            board.contains(&target)
                && !board.is_snake_body(&target)
                && !state.you.body.contains(&target)
        })
        .collect()
}

/// Cells reachable from where the move lands, without crossing the board
/// edge, a snake's body or a hazard.
fn space_after(board: &Board, head: Coord, mv: Move) -> usize {
    let start = mv.step(head);
    let mut visited = HashSet::from([start]);
    let mut frontier = vec![start];

    while let Some(cell) = frontier.pop() {
        for neighbour in Move::ALL.map(|m| m.step(cell)) {
            if board.is_free(&neighbour) && visited.insert(neighbour) {
                frontier.push(neighbour);
            }
        }
    }

    visited.len()
}

/// The safe moves with the most space, in fixed move order.
fn best_moves(board: &Board, head: Coord, safe: &[Move]) -> Vec<Move> {
    let spaces: Vec<(Move, usize)> = safe
        .iter()
        .map(|&mv| (mv, space_after(board, head, mv)))
        .collect();
    debug!("Space: {spaces:?}");

    let most = spaces.iter().map(|&(_, space)| space).max().unwrap_or(0);
    spaces
        .into_iter()
        .filter(|&(_, space)| space == most)
        .map(|(mv, _)| mv)
        .collect()
}

/// The food with the smallest Manhattan distance from the head.
fn nearest_food(board: &Board, head: Coord) -> Option<Coord> {
    board
        .food
        .iter()
        .min_by_key(|food| food.manhattan_distance(&head))
        .copied()
}

/// Picks among the best moves: the one whose landing cell is closest to the
/// nearest food, with remaining ties (or no food at all) settled by fixed
/// move order.
fn choose_among_best(best: &[Move], head: Coord, food: Option<Coord>) -> Move {
    let distance_to_food = |mv: &Move| food.map(|food| food.manhattan_distance(&mv.step(head)));
    // `min_by_key` keeps the first of equal keys, so fixed order breaks ties.
    best.iter()
        .copied()
        .min_by_key(distance_to_food)
        .unwrap_or(Move::Up)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Battlesnake, Board, Game};
    use std::collections::HashMap;

    fn c(x: i32, y: i32) -> Coord {
        Coord { x, y }
    }

    /// Test-only builder for a game state seen from our snake.
    struct BoardBuilder {
        width: i32,
        height: i32,
        you: Vec<Coord>,
        opponents: Vec<Vec<Coord>>,
        food: Vec<Coord>,
        hazards: Vec<Coord>,
    }

    fn board(width: i32, height: i32) -> BoardBuilder {
        BoardBuilder {
            width,
            height,
            you: vec![],
            opponents: vec![],
            food: vec![],
            hazards: vec![],
        }
    }

    fn snake(id: &str, body: Vec<Coord>) -> Battlesnake {
        Battlesnake {
            id: id.to_string(),
            name: id.to_string(),
            health: 100,
            head: body[0],
            length: body.len() as i32,
            body,
            latency: "0".to_string(),
            shout: None,
        }
    }

    impl BoardBuilder {
        fn you(mut self, body: &[(i32, i32)]) -> Self {
            self.you = body.iter().map(|&(x, y)| c(x, y)).collect();
            self
        }

        fn opponent(mut self, body: &[(i32, i32)]) -> Self {
            self.opponents
                .push(body.iter().map(|&(x, y)| c(x, y)).collect());
            self
        }

        fn food(mut self, x: i32, y: i32) -> Self {
            self.food.push(c(x, y));
            self
        }

        fn hazard(mut self, x: i32, y: i32) -> Self {
            self.hazards.push(c(x, y));
            self
        }

        fn state(self) -> GameState {
            let mut snakes = vec![snake("you", self.you.clone())];
            for (i, body) in self.opponents.into_iter().enumerate() {
                snakes.push(snake(&format!("opponent-{i}"), body));
            }
            GameState {
                game: Game {
                    id: "game".to_string(),
                    ruleset: HashMap::new(),
                    timeout: 500,
                },
                turn: 0,
                board: Board {
                    width: self.width,
                    height: self.height,
                    food: self.food,
                    snakes,
                    hazards: self.hazards,
                },
                you: snake("you", self.you),
            }
        }

        fn chosen_move(self) -> Move {
            choose_move(&self.state())
        }
    }

    #[test]
    fn never_leaves_the_board_from_any_corner() {
        // Body runs along one edge from each corner, leaving one safe move.
        let cases = [
            (&[(0, 0), (1, 0), (2, 0)], Move::Up),
            (&[(0, 4), (0, 3), (0, 2)], Move::Right),
            (&[(4, 4), (3, 4), (2, 4)], Move::Down),
            (&[(4, 0), (4, 1), (4, 2)], Move::Left),
        ];
        for (body, expected) in cases {
            assert_eq!(board(5, 5).you(body).chosen_move(), expected, "{body:?}");
        }
    }

    #[test]
    fn never_leaves_the_board_from_any_edge() {
        // Head mid-edge with no food, and every move ahead of the off-board
        // one in fixed order blocked, so only the edge check stops it.
        let top = board(5, 5).you(&[(2, 4), (2, 3), (2, 2)]);
        let bottom = board(5, 5).you(&[(2, 0), (2, 1), (2, 2)]);
        let left = board(5, 5)
            .you(&[(0, 2), (0, 3), (0, 4)])
            .opponent(&[(0, 1), (0, 0)]);
        let right = board(5, 5)
            .you(&[(4, 2), (4, 3), (4, 4)])
            .opponent(&[(4, 1), (4, 0)]);

        assert_eq!(top.chosen_move(), Move::Left);
        assert_eq!(bottom.chosen_move(), Move::Left);
        assert_eq!(left.chosen_move(), Move::Right);
        assert_eq!(right.chosen_move(), Move::Left);
    }

    #[test]
    fn never_moves_into_its_own_body() {
        // Head in the middle, body wraps above and to the right.
        let chosen = board(5, 5)
            .you(&[(2, 2), (2, 3), (3, 3), (3, 2), (3, 1)])
            .food(4, 4)
            .chosen_move();
        assert!(matches!(chosen, Move::Down | Move::Left));
    }

    #[test]
    fn never_moves_into_an_opponents_body() {
        // Food lies to the right, but an opponent's body blocks that way.
        let chosen = board(5, 5)
            .you(&[(0, 2), (0, 1), (0, 0)])
            .opponent(&[(1, 2), (2, 2), (3, 2)])
            .food(4, 2)
            .chosen_move();
        assert_eq!(chosen, Move::Up);
    }

    #[test]
    fn picks_more_space_over_a_pocket_smaller_than_itself_even_with_food_in_it() {
        // Moving left enters a three-cell pocket holding the food; the snake is
        // four long, so it keeps to the open side instead.
        let chosen = board(5, 5)
            .you(&[(2, 3), (2, 2), (2, 1), (2, 0)])
            .opponent(&[(1, 2), (0, 2), (0, 1)])
            .opponent(&[(1, 4)])
            .food(0, 3)
            .chosen_move();
        assert_ne!(chosen, Move::Left);
    }

    #[test]
    fn takes_the_move_towards_food_when_space_ties() {
        let chosen = board(7, 7)
            .you(&[(3, 3), (3, 2), (3, 1)])
            .food(0, 3)
            .chosen_move();
        assert_eq!(chosen, Move::Left);
    }

    #[test]
    fn follows_the_fixed_move_order_when_there_is_no_food() {
        // Up is blocked by its own body; down, left and right tie on space.
        let chosen = board(5, 5).you(&[(2, 2), (2, 3), (2, 4)]).chosen_move();
        assert_eq!(chosen, Move::Down);
    }

    #[test]
    fn moves_towards_food_on_the_y_axis_when_x_is_aligned() {
        // Food directly below; up, down and left tie on space.
        let chosen = board(7, 7)
            .you(&[(3, 3), (4, 3), (5, 3)])
            .food(3, 0)
            .chosen_move();
        assert_eq!(chosen, Move::Down);
    }

    #[test]
    fn breaks_ties_in_fixed_move_order_when_food_is_diagonal() {
        // Food is up and to the left: up and left tie on space and on distance
        // to the nearest food, so up wins, on every call.
        let state = board(7, 7)
            .you(&[(3, 3), (3, 2), (3, 1)])
            .food(0, 6)
            .state();
        assert_eq!(choose_move(&state), Move::Up);
        assert_eq!(choose_move(&state), Move::Up);
    }

    #[test]
    fn moves_up_when_there_are_no_safe_moves() {
        let chosen = board(3, 3)
            .you(&[(0, 0), (0, 1), (1, 1), (1, 0)])
            .opponent(&[(2, 2), (2, 1), (2, 0)])
            .food(2, 2)
            .chosen_move();
        assert_eq!(chosen, Move::Up);
    }

    #[test]
    fn hazards_count_against_space() {
        // Both up and down are open; hazards fill the cells above, so going
        // down reaches more space even though the food is above.
        let chosen = board(3, 5)
            .you(&[(1, 2), (2, 2), (2, 1)])
            .hazard(0, 4)
            .hazard(1, 4)
            .hazard(2, 4)
            .hazard(0, 3)
            .hazard(2, 3)
            .food(1, 3)
            .chosen_move();
        assert_eq!(chosen, Move::Down);
    }
}
