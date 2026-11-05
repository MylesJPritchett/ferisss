# ferisss

A [Battlesnake](https://play.battlesnake.com) written in Rust: a small HTTP server that gets the game state each turn and decides where to move. It avoids trapping itself by measuring the open space behind each possible move.

<!--
  SCREENSHOT: record a GIF of a local game (see "Play a local game" below), 10–15 seconds.
  Best moment to capture: the snake turning away from nearby food because that move
  leads into a small enclosed pocket. A solo game on an 11x11 board, around turn 60–120
  once the snake is long, shows this most often. Save as docs/demo.gif and replace this comment with:
  ![ferisss in a local game](docs/demo.gif)
-->

<!-- LIVE: if the snake is registered on play.battlesnake.com, link its profile page here. -->

## What it does

- Never moves off the board or into its own body or another snake's body.
- Each turn, measures how much open board can be reached from each safe move, and only chooses among the moves with the most space.
- Heads for the nearest food when that keeps it in the most space, and gives up the food when it doesn't.
- Runs as a stateless server, so it can play several games at the same time.

ferisss started from the official [Battlesnake Rust starter](https://github.com/BattlesnakeOfficial/starter-snake-rust), which provides the Rocket server and the API types. Everything in `src/logic.rs` beyond the starter's boundary check is my own work (body and opponent avoidance, flood fill, food selection), as are the Docker image and the CI deployment.

## How it works

The Battlesnake engine sends `POST /move` with the full board as JSON once per turn, and the snake must reply within the game's timeout (500 ms by default). `src/main.rs` deserialises the board into `GameState` with serde and hands it to `logic::get_move`, which runs three passes:

1. **Filter.** Start with up/down/left/right and remove any move that leaves the board or lands on a snake body (its own or another snake's).
2. **Score by space.** For each remaining move, run a breadth-first flood fill (`flood_fill_count`) from the target cell. Every cell it can reach without crossing a wall, a snake body or a hazard adds to that move's count.
3. **Pick.** Keep only the moves with the highest count. If the move toward the nearest food (by Manhattan distance) is among them, take it. Otherwise take one of the best-space moves.

Some decisions worth calling out:

- **Space first, food second.** Checking only one step ahead isn't enough. A cell can be safe this turn and still lead into a pocket smaller than the snake, and a snake that goes straight for food will walk into those pockets. Ranking moves by the size of the area they lead to prevents that. Food only breaks ties, so the snake chooses survival over growth.
- **Flood fill rather than pathfinding.** A shortest path to food (there's a commented-out BFS sketch in `get_move`) tells you how to reach the food, not whether you can get out afterwards. Counting reachable cells answers the question that actually kills snakes, and it's cheap on an 11×11 board: at most 121 cells for each of four candidate moves. The visited set is a `HashSet<Coord>`, which is why `Coord` derives `Hash`/`Eq`.
- **Container deploy.** A two-stage Dockerfile compiles in `rust:latest` and copies only the release binary into `ubuntu:24.04`. On every push to `main`, a GitHub Actions workflow builds the image, pushes it to Docker Hub, and pulls it onto a DigitalOcean Droplet over SSH.

### Known limitations

- If the board has no food, `get_move` panics on `unwrap()` and the request returns a 500. The engine then moves the snake for it.
- Snake tails are treated as permanently blocked, even though they move away next turn. Hazards are treated as walls. Both make the flood fill pessimistic.
- It doesn't predict opponents' heads, so it can walk into a head-to-head collision with a longer snake.
- When several moves tie, the choice depends on `HashMap` iteration order rather than an explicit rule.
- The deploy workflow pulls the new image but doesn't restart the running container.
- There are no automated tests yet.

## Stack

Rust (2018 edition), [Rocket 0.5](https://rocket.rs) for HTTP, serde/serde_json for the API types, env_logger for logging, Docker, GitHub Actions, and a DigitalOcean Droplet.

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
docker run --rm -p 8000:8000 -e ROCKET_ADDRESS=0.0.0.0 ferisss
```

`ROCKET_ADDRESS` is needed because the runtime image contains only the binary, not `Rocket.toml`, so Rocket otherwise binds to `127.0.0.1` inside the container.

### Play a local game

Install the [Battlesnake CLI](https://github.com/BattlesnakeOfficial/rules/tree/main/cli), start the server, then:

```sh
battlesnake play -W 11 -H 11 --name ferisss --url http://localhost:8000 -g solo --browser
```

### Tests

```sh
cargo test
```

The crate has no tests yet, so this only checks that it compiles.
