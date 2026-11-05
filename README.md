# ferisss

[![CI](https://github.com/MylesJPritchett/ferisss/actions/workflows/ci.yml/badge.svg)](https://github.com/MylesJPritchett/ferisss/actions/workflows/ci.yml)

A [Battlesnake](https://play.battlesnake.com) written in Rust: a small HTTP server that gets the game state each turn and decides where to move. It avoids trapping itself by measuring the open space behind each possible move.

<!--
  SCREENSHOT: record a GIF of a local game (see "Play a local game" below), 10–15 seconds.
  Best moment to capture: the snake turning away from nearby food because that move
  leads into a small enclosed pocket. A solo game on an 11x11 board, around turn 60–120
  once the snake is long, shows this most often. Save as docs/demo.gif and replace this comment with:
  ![ferisss in a local game](docs/demo.gif)
-->

<!-- LIVE: if the snake is registered on play.battlesnake.com, link its profile page here. -->

The snake isn't currently deployed. Deploys are manual (see "Container deploy" below).

## What it does

- Never moves off the board or into its own body or another snake's body.
- Each turn, measures how much open board can be reached from each safe move, and only chooses among the moves with the most space.
- Among the moves with the most space, heads for the nearest food, and gives up the food when that would cost space.
- Breaks any remaining tie in a fixed order (up, down, left, right), so the same board always gets the same move.
- Runs as a stateless server, so it can play several games at the same time.

ferisss started from the official [Battlesnake Rust starter](https://github.com/BattlesnakeOfficial/starter-snake-rust), which provides the Rocket server and the API types. Everything in `src/logic.rs` beyond the starter's boundary check is my own work (body and opponent avoidance, flood fill, food selection, tie-breaking), as are the module split, the tests, the Docker image and the CI and deploy workflows.

## How it works

The Battlesnake engine sends `POST /move` with the full board as JSON once per turn, and the snake must reply within the game's timeout (500 ms by default). `src/main.rs` deserialises the board into `GameState` (the API types live in `src/api.rs`) with serde and hands it to `logic::get_move`, which runs three passes:

1. **Filter.** Start with up/down/left/right and remove any move that leaves the board or lands on a snake body (its own or another snake's). If nothing is left, it moves up.
2. **Score by space.** For each remaining move, run a flood fill (`space_after`) from the target cell. Every cell it can reach without crossing a wall, a snake body or a hazard adds to that move's count.
3. **Pick.** Keep only the moves with the highest count. Among those, take the one whose target cell is closest (by Manhattan distance) to the nearest food. Any tie left over, or every tie when there's no food, goes to the first move in the fixed order up, down, left, right.

Some decisions worth calling out:

- **Space first, food second.** Checking only one step ahead isn't enough. A cell can be safe this turn and still lead into a pocket smaller than the snake, and a snake that goes straight for food will walk into those pockets. Ranking moves by the size of the area they lead to prevents that. Food only breaks ties, so the snake chooses survival over growth.
- **Deterministic ties.** Moves are always considered in the same order, and the first of any equal options wins. The same board always produces the same move, which keeps games reproducible and the tests exact.
- **Flood fill rather than pathfinding.** A shortest path to food tells you how to reach the food, not whether you can get out afterwards. Counting reachable cells answers the question that actually kills snakes, and it's cheap on an 11×11 board: at most 121 cells for each of four candidate moves. The visited set is a `HashSet<Coord>`, which is why `Coord` derives `Hash`/`Eq`.
- **Container deploy.** A two-stage Dockerfile compiles in `rust:1-bookworm` and copies only the release binary and `Rocket.toml` into `debian:bookworm-slim`, where it runs as a non-root user. The deploy workflow is run by hand (`workflow_dispatch`): it builds the image, pushes it to Docker Hub, then over SSH pulls it onto a DigitalOcean Droplet and replaces the running container. A separate CI workflow runs `cargo fmt --check`, `cargo clippy` with warnings as errors, and `cargo test` on every push and pull request.

### Known limitations

- Snake tails are treated as permanently blocked, even though they move away next turn. Hazards are treated as walls when measuring space. Both make the flood fill pessimistic.
- It doesn't predict opponents' heads, so it can walk into a head-to-head collision with a longer snake.

## Stack

Rust (2024 edition), [Rocket 0.5](https://rocket.rs) for HTTP, serde/serde_json for the API types, env_logger for logging, Docker, GitHub Actions, and a DigitalOcean Droplet.

## Running locally

Requires a stable Rust toolchain.

```sh
cargo run          # serves on http://0.0.0.0:8000 (set PORT to change)
curl localhost:8000/
# {"apiversion":"1","author":"MylesJPritchett","color":"#FF5F1F","head":"default","tail":"default"}
```

With Docker:

```sh
docker build -t ferisss .
docker run --rm -p 8000:8000 ferisss
```

The image includes `Rocket.toml`, which binds to `0.0.0.0`, so no extra environment is needed.

### Play a local game

Install the [Battlesnake CLI](https://github.com/BattlesnakeOfficial/rules/tree/main/cli), start the server, then:

```sh
battlesnake play -W 11 -H 11 --name ferisss --url http://localhost:8000 -g solo --browser
```

### Tests

```sh
cargo test
```

The move tests in `src/logic.rs` build small boards with a test-only builder and check the move `choose_move` picks: never leaving the board from any corner or edge, never entering its own or an opponent's body, avoiding a pocket smaller than itself even with food in it, heading for food when space ties, treating hazards as lost space, and breaking ties in the fixed order. The server tests in `src/main.rs` check the `Server` header and that `PORT` overrides the default port of 8000. CI runs the same suite alongside `cargo fmt --check` and `cargo clippy`.
