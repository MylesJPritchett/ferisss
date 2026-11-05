# ferisss

How the snake decides its next move each turn.

## Language

**Move**:
One of the four directions the snake's head can go this turn: up, down, left or right.
_Avoid_: Direction, step

**Safe move**:
A move that doesn't leave the board or land on any snake's body this turn.
_Avoid_: Valid move, legal move

**Space**:
The number of cells reachable from the cell a move lands on, without crossing the board edge, a snake's body or a hazard.
_Avoid_: Flood fill count, area, room

**Best moves**:
The safe moves with the most space.
_Avoid_: Top moves, optimal moves

**Nearest food**:
The food with the smallest Manhattan distance from the snake's head.
_Avoid_: Closest food, target
