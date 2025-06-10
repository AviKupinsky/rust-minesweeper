//! Entry point for the Minesweeper game.
//!
//! This file sets up the game window, board size, and launches the main application loop.
//! It imports all core modules and initializes the MinesweeperApp with the chosen board parameters.
//! The window size is automatically configured to fit the board and UI.

mod board;
mod gui;
use gui::MinesweeperApp;
mod gui_animation;
mod gui_board;
mod gui_popup;
mod gui_ui;
mod particle;


// Medium
const BOARD_WIDTH: usize = 16;
const BOARD_HEIGHT: usize = 16;
const CELL_SIZE: f32 = 36.0; // Smaller cells
const MINES: usize = 40;
const TOP_BAR_HEIGHT: f32 = 60.0;


// This function sets the window size to exactly fit the board and top bar
fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: macroquad::miniquad::conf::Conf {
            window_title: "Minesweeper".to_owned(),
            window_width: (BOARD_WIDTH as f32 * CELL_SIZE) as i32,
            window_height: (BOARD_HEIGHT as f32 * CELL_SIZE + TOP_BAR_HEIGHT) as i32,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = MinesweeperApp::new(BOARD_WIDTH, BOARD_HEIGHT, MINES);
    app.run().await;
}
