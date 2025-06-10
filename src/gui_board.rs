//! Board and cell logic for Minesweeper.
//!
//! This module contains all functions and methods related to drawing the board and cells,
//! handling mouse clicks on the board, uncovering cells, flagging, win/loss checks, and
//! other board-specific logic. Animation and UI logic are handled in their respective modules.

use super::MinesweeperApp;
use crate::board::*;
use crate::gui::GameState;
use crate::particle::*;
use macroquad::audio::*;
use macroquad::prelude::*;

// use crate::gui_animation::*;

// --- Animation and drawing constants ---
//
// These constants define the appearance and layout of the Minesweeper board and cells.
// Adjust these values to change the board's look and feel.
//
const TOP_BAR_HEIGHT: f32 = 60.0;
const COVERED_COLOR_EVEN: Color = Color::from_rgba(255, 180, 60, 255);
const COVERED_COLOR_ODD: Color = Color::from_rgba(255, 200, 100, 255);
const UNCOVERED_COLOR_EVEN: Color = Color::from_rgba(195, 195, 195, 255);
const UNCOVERED_COLOR_ODD: Color = Color::from_rgba(225, 225, 225, 255);
const NUMBER_FONT_SCALE: f32 = 0.8; // Proportion of cell size for number font
const NUMBER_TEXT_Y_OFFSET: f32 = -4.0; // Vertical adjustment for centering text
const FLAG_ICON_SCALE: f32 = 0.7;
const FLAG_XY_OFFSET: f32 = 6.0;
const FLAG_LINE_WIDTH: f32 = 4.0;
const MINE_ICON_SCALE: f32 = 0.7;

// All these are methods for MinesweeperApp
impl MinesweeperApp {
    /// Draws the Minesweeper board, including all cells and their contents.
    pub fn draw_board(
        &mut self,
        cell_size: f32,
        flag_texture: &Texture2D,
        mine_texture: &Texture2D,
        win_sound: &Sound,
    ) {
        for row in 0..self.board().height() {
            for col in 0..self.board().width() {
                let x = col as f32 * cell_size;
                let y = row as f32 * cell_size + TOP_BAR_HEIGHT;
                let is_even = (row + col) % 2 == 0;
                let covered_color = if is_even {
                    COVERED_COLOR_EVEN
                } else {
                    COVERED_COLOR_ODD
                };
                let uncovered_color = if is_even {
                    UNCOVERED_COLOR_EVEN
                } else {
                    UNCOVERED_COLOR_ODD
                };
                let cell_state = self
                    .board()
                    .cell_state(row, col)
                    .unwrap_or(CellState::Covered);
                let cell = self.board().cell(row, col).unwrap_or(Cell::Empty);

                // Handle wave/flood-fill animation for this cell
                if self.handle_wave_animation(row, col, cell_size, win_sound) {
                    continue;
                }
                // Handle pop animation for this cell
                if self.handle_pop_animation(row, col, cell, x, y, cell_size, uncovered_color) {
                    continue;
                }

                // Draw cell background and border
                let bg_color = match cell_state {
                    CellState::Covered | CellState::Flagged => covered_color,
                    CellState::Uncovered => uncovered_color,
                };
                draw_rectangle(x, y, cell_size, cell_size, bg_color);
                draw_rectangle_lines(x, y, cell_size, cell_size, 2.0, DARKGRAY);

                // Draw the cell content (flag, mine, number, or nothing)
                self.draw_cell_content(
                    cell_state,
                    cell,
                    row,
                    col,
                    x,
                    y,
                    cell_size,
                    flag_texture,
                    mine_texture,
                );
            }
        }
    }

    /// Draws the content inside a cell based on its state and value.
    fn draw_cell_content(
        &self,
        cell_state: CellState,
        cell: Cell,
        row: usize,
        col: usize,
        x: f32,
        y: f32,
        cell_size: f32,
        flag_texture: &Texture2D,
        mine_texture: &Texture2D,
    ) {
        match cell_state {
            CellState::Covered => {
                // Covered cell: nothing to draw inside
            }
            CellState::Flagged => {
                // Draw the flag icon centered in the cell
                draw_texture_ex(
                    flag_texture,
                    x + (cell_size - cell_size * FLAG_ICON_SCALE) / 2.0,
                    y + (cell_size - cell_size * FLAG_ICON_SCALE) / 2.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(
                            cell_size * FLAG_ICON_SCALE,
                            cell_size * FLAG_ICON_SCALE,
                        )),
                        ..Default::default()
                    },
                );
                // If the game is over and this is a wrong flag, draw a red X over the flag
                if (self.state() == GameState::GameOver || self.state() == GameState::Lost)
                    && self.wrong_flags().contains(&(row, col))
                {
                    let x1 = x + FLAG_XY_OFFSET;
                    let y1 = y + FLAG_XY_OFFSET;
                    let x2 = x + cell_size - FLAG_XY_OFFSET;
                    let y2 = y + cell_size - FLAG_XY_OFFSET;
                    draw_line(x1, y1, x2, y2, FLAG_LINE_WIDTH, RED);
                    draw_line(x1, y2, x2, y1, FLAG_LINE_WIDTH, RED);
                }
            }
            CellState::Uncovered => {
                match cell {
                    Cell::Mine => {
                        // Draw the mine icon centered in the cell
                        draw_texture_ex(
                            mine_texture,
                            x + (cell_size - cell_size * MINE_ICON_SCALE) / 2.0,
                            y + (cell_size - cell_size * MINE_ICON_SCALE) / 2.0,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(Vec2::new(
                                    cell_size * MINE_ICON_SCALE,
                                    cell_size * MINE_ICON_SCALE,
                                )),
                                ..Default::default()
                            },
                        );
                    }
                    Cell::Number(n) => {
                        // Draw the number in the center of the cell
                        self.draw_cell_number(
                            n,
                            x + cell_size / 2.0,
                            y + cell_size / 2.0,
                            cell_size,
                        );
                    }
                    Cell::Empty => {
                        // Empty uncovered cell: nothing to draw inside
                    }
                }
            }
        }
    }

    /// Draws a cell number with classic Minesweeper color and proper centering.
    pub fn draw_cell_number(&self, n: u8, cx: f32, cy: f32, cell_size: f32) {
        let label = n.to_string();
        let text_color = match n {
            1 => BLUE,
            2 => GREEN,
            3 => RED,
            4 => DARKBLUE,
            5 => MAROON,
            6 => DARKGREEN,
            7 => BLACK,
            8 => GRAY,
            _ => BLACK,
        };
        let font_size = cell_size * NUMBER_FONT_SCALE;
        let text_dim = measure_text(&label, None, font_size as u16, 1.0);
        draw_text(
            &label,
            cx - text_dim.width / 2.0,
            cy + text_dim.height / 2.0 + NUMBER_TEXT_Y_OFFSET,
            font_size,
            text_color,
        );
    }

    /// Converts mouse position to (row, col) if within the board, else returns None.
    pub fn mouse_to_cell(&self, cell_size: f32) -> Option<(usize, usize)> {
        let (mx, my) = mouse_position();
        if my < TOP_BAR_HEIGHT {
            return None;
        }
        let col = (mx / cell_size) as usize;
        let row = ((my - TOP_BAR_HEIGHT) / cell_size) as usize;
        if row < self.board().height() && col < self.board().width() {
            Some((row, col))
        } else {
            None
        }
    }

    /// Handles all logic for a left mouse click on the board.
    /// This includes starting the timer, placing mines on first click,
    /// handling mine clicks, empty cell clicks (flood fill), and number cell clicks.
    pub fn handle_left_click(
        &mut self,
        row: usize,
        col: usize,
        cell_size: f32,
        mine_reveal_timer: &mut f32,
        bomb_sound: &Sound,
        flip_sound: &Sound,
        wave_sound: &Sound,
        win_sound: &Sound,
    ) {
        // On the first click, start the timer, place mines, and set the game state to running
        if self.state() == GameState::NotStarted {
            self.set_start_time(get_time());
            self.board_mut().place_mines_avoiding(row, col);
            self.board_mut().calculate_numbers();
            self.set_state(GameState::Running);
        }
        // Handle what was clicked
        match self.board().cell(row, col) {
            Some(Cell::Mine) => {
                self.handle_mine_click(row, col, cell_size, mine_reveal_timer, bomb_sound)
            }
            Some(Cell::Empty) => {
                self.handle_empty_click(row, col, cell_size, wave_sound, win_sound)
            }
            _ => self.handle_number_click(row, col, cell_size, flip_sound, win_sound),
        }
    }

    /// Handles all logic for a right mouse click on the board (flag/unflag).
    pub fn handle_right_click(
        &mut self,
        row: usize,
        col: usize,
        flag_sound: &Sound,
        remove_flag_sound: &Sound,
    ) {
        match self.board().cell_state(row, col) {
            Some(CellState::Covered) => {
                self.board_mut().flag_cell(row, col);
                // Play flag sound when flag is placed
                if self.sound() {
                    play_sound(
                        flag_sound,
                        PlaySoundParams {
                            looped: false,
                            volume: 0.6,
                        },
                    );
                }
            }
            Some(CellState::Flagged) => {
                self.board_mut().unflag_cell(row, col);
                if self.sound() {
                    play_sound(
                        remove_flag_sound,
                        PlaySoundParams {
                            looped: false,
                            volume: 0.6,
                        },
                    );
                }
            }
            _ => {}
        }
    }

    /// Handles logic for clicking an empty cell (starts flood fill animation).
    fn handle_empty_click(
        &mut self,
        row: usize,
        col: usize,
        cell_size: f32,
        wave_sound: &Sound,
        win_sound: &Sound,
    ) {
        if self.sound() {
            play_sound(
                wave_sound,
                PlaySoundParams {
                    looped: false,
                    volume: 0.5,
                },
            );
        }
        let revealed = self.board_mut().flood_fill_wave(row, col);
        for &(r, c, dist) in &revealed {
            let delay = dist as f32 * 0.05;
            self.wave_timers_mut()[r][c] = Some(delay);
        }
        self.check_win(cell_size, win_sound);
    }

    /// Handles logic for clicking a number cell (uncover and pop animation).
    fn handle_number_click(
        &mut self,
        row: usize,
        col: usize,
        cell_size: f32,
        flip_sound: &Sound,
        win_sound: &Sound,
    ) {
        if self.sound() {
            play_sound(
                flip_sound,
                PlaySoundParams {
                    looped: false,
                    volume: 0.5,
                },
            );
        }
        self.board_mut().uncover_cell(row, col);
        self.pop_timers_mut()[row][col] = Some(0.0);
        self.check_win(cell_size, win_sound);
    }

    /// Handles logic for clicking a mine cell.
    fn handle_mine_click(
        &mut self,
        row: usize,
        col: usize,
        cell_size: f32,
        mine_reveal_timer: &mut f32,
        bomb_sound: &Sound,
    ) {
        if self.sound() {
            play_sound(
                bomb_sound,
                PlaySoundParams {
                    looped: false,
                    volume: 0.7,
                },
            ); // Play bomb sound
        }
        self.board_mut().uncover_cell(row, col);
        spawn_particles(
            &mut self.particles_mut(),
            row,
            col,
            cell_size,
            true,
            None,
            TOP_BAR_HEIGHT,
        );
        self.spawn_shockwave(row, col, cell_size);

        // Build a new queue of mines to reveal (excluding flagged and the one just clicked).
        // We use a temporary variable to avoid borrowing self.mine_reveal_queue and self.board at the same time,
        // which would cause a Rust borrow checker error.
        let mut new_queue: Vec<(usize, usize, bool)> = self
            .board()
            .mine_positions()
            .iter()
            .cloned()
            .filter(|&(r2, c2)| {
                self.board().cell_state(r2, c2) != Some(CellState::Flagged)
                    && !(r2 == row && c2 == col)
            })
            .map(|(r, c)| (r, c, true))
            .collect();

        // Add wrongly flagged cells to the queue.
        // Again, we collect into a temporary variable to avoid borrow checker issues.
        let wrong_flags: Vec<_> = (0..self.board().height())
            .flat_map(|r| (0..self.board().width()).map(move |c| (r, c)))
            .filter(|&(r, c)| {
                self.board().cell_state(r, c) == Some(CellState::Flagged)
                    && self.board().cell(r, c) != Some(Cell::Mine)
            })
            .map(|(r, c)| (r, c, false))
            .collect();

        // Extend the new_queue with wrong flags, then assign it to mine_reveal_queue.
        // This ensures all borrows are finished before mutably borrowing self.mine_reveal_queue.
        new_queue.extend(wrong_flags);
        let queue = self.mine_reveal_queue_mut();
        *queue = new_queue;

        // Shuffle the mine_reveal_queue in a pseudo-random order using a hash of the cell coordinates and the current time.
        // This gives a different reveal order each game over, without needing an external random crate.
        let now = get_time();
        self.mine_reveal_queue_mut().sort_by(|a, b| {
            let hash_a = ((a.0 as f64 * 13.37 + a.1 as f64 * 42.42 + now) * 1000.0) as i64;
            let hash_b = ((b.0 as f64 * 13.37 + b.1 as f64 * 42.42 + now) * 1000.0) as i64;
            hash_a.cmp(&hash_b)
        });

        *mine_reveal_timer = 0.0;
        self.set_end_time(Some(get_time()));
        self.set_state(GameState::GameOver); // Fill the queue with all other mines to reveal (except flagged and the one just clicked)
                                             // self.wrong_flags.clear();
    }

    pub fn check_win(&mut self, cell_size: f32, win_sound: &Sound) {
        // Checks if the player has won the game by uncovering all non-mine cells.
        for row in 0..self.board().height() {
            for col in 0..self.board().width() {
                if self.board().cell(row, col) != Some(Cell::Mine)
                    && self.board().cell_state(row, col) != Some(CellState::Uncovered)
                {
                    return; // Not won yet, exit early
                }
            }
        }
        // If we get here, all non-mine cells are uncovered
        self.set_end_time(Some(get_time()));
        self.set_state(GameState::Won);
        if self.sound() {
            play_sound(
                win_sound,
                PlaySoundParams {
                    looped: false,
                    volume: 0.8,
                },
            );
        }
        // Get the board width before mutably borrowing self for particles.
        // This avoids Rust's borrow checker error by ensuring the immutable borrow ends
        // before the mutable borrow of self.particles begins.
        let board_width = self.board().width();
        spawn_confetti(&mut self.particles_mut(), board_width, cell_size);
    }
}
