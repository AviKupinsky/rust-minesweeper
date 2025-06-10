# Minesweeper in Rust

A classic Minesweeper game implemented in Rust using Macroquad for graphics and input.

## Features

- Classic Minesweeper gameplay with three board sizes: Small, Medium (default), and Large
- Responsive GUI with sound toggle, timer, and flag counter
- Robust game logic with edge-case handling (first click never hits a mine, win/loss detection, etc.)
- Comprehensive test suite covering board logic, UI state, and user interactions

## How Minesweeper Works

Minesweeper is a logic puzzle game. The board is a grid of covered cells, some of which contain hidden mines.  
Your goal is to uncover all cells that do **not** contain mines, using logic and deduction.

- **Uncovering a cell:**  
  - If it contains a mine, you lose.
  - If it does not, it shows a number (how many adjacent cells have mines) or is empty (no adjacent mines).
  - Uncovering an empty cell automatically uncovers all adjacent empty cells.
- **Flagging:**  
  - You can flag cells you suspect contain mines.
  - The game tracks the number of flags and mines remaining.
- **Winning:**  
  - You win by uncovering all non-mine cells.
- **Losing:**  
  - You lose if you uncover a mine.

## Project Structure

- `src/`
  - `main.rs` — Entry point; sets up the game window and launches the Minesweeper app.
  - `lib.rs` — Library root; exposes modules and re-exports types for use in the app and tests.
  - `board.rs` — Core game logic: board state, cell logic, mine placement, uncovering, flagging, etc.
  - `gui.rs` — Main GUI logic and app state management.
  - `gui_board.rs` — Handles rendering the Minesweeper board in the GUI.
  - `gui_ui.rs` — Handles UI elements (buttons, menus, etc.).
  - `gui_popup.rs` — Handles popups (e.g., game over, win dialogs).
  - `gui_animation.rs` — Handles animations for cell reveals, effects, etc.
  - `particle.rs` — Particle effects for visual feedback.
- `assets/` — Images, sounds, and other resources used by the game.
- `tests/`
  - `Minesweeper_tests.rs` — Comprehensive test suite for board logic and app-level behavior.
- `doc/` — Documentation or design notes for the project.
- `Cargo.toml` — Rust package manifest (lists dependencies like Macroquad).
- `Cargo.lock` — Locked dependency versions.
- `.gitignore` — Files and folders to ignore in git (target/, *.rs.bk, etc.)
- `README.md` — This file.
- `LICENSE` — License for your project (MIT recommended).

> **Note:**  
> Some files (like `gui_animation.rs`, `particle.rs`, or `assets/`) are included as placeholders or for future expansion.  
> You can continue to add or extend these files as you develop new features, animations, or visual effects for your Minesweeper game.

## Getting Started

1. **Install Rust:**  
   [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

2. **Download dependencies:**  
   In your project folder, run:
   ```sh
   cargo fetch
   ```

3. **Run the game:**  
   ```sh
   cargo run
   ```


4. **Run tests:**  
   ```sh
   cargo test --test Minesweeper_tests
   ```




Enjoy playing and hacking on Minesweeper in Rust!
