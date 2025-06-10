//! Popup and endgame UI logic for Minesweeper.
//!
//! This module contains all functions and methods related to drawing popups for win/loss states,
//! handling the "Play Again" button, and managing endgame UI logic. Board logic, animation, and
//! general UI drawing are handled in other modules.

use super::MinesweeperApp;
use crate::gui::GameState;
use macroquad::audio::*;
use macroquad::prelude::*;

// --- Popup and UI Constants ---
//
// These constants control the appearance and layout of the endgame popup UI.
//
const TOP_BAR_HEIGHT: f32 = 60.0;
const POPUP_WIDTH: f32 = 320.0;
const POPUP_HEIGHT: f32 = 140.0;
const POPUP_BORDER_WIDTH: f32 = 4.0;
const POPUP_BG_COLOR: Color = Color::from_rgba(30, 30, 30, 240);
const POPUP_MSG_FONT_SIZE: f32 = 28.0;
const POPUP_MSG_Y_OFFSET: f32 = 60.0;
const POPUP_BTN_WIDTH: f32 = 120.0;
const POPUP_BTN_HEIGHT: f32 = 36.0;
const POPUP_BTN_Y_MARGIN: f32 = 16.0;
const POPUP_BTN_LABEL_FONT_SIZE: u16 = 22;
const POPUP_BTN_LABEL_Y_OFFSET: f32 = -4.0;
const POPUP_BTN_LABEL: &str = "Play Again";

impl MinesweeperApp {
    /// Draws a centered popup with a message and a "Play Again" button.
    pub fn draw_popup(&mut self, cell_size: f32, border_color: Color, msg: &str) -> bool {
        let popup_x = (self.board().width() as f32 * cell_size - POPUP_WIDTH) / 2.0;
        let popup_y =
            (self.board().height() as f32 * cell_size + TOP_BAR_HEIGHT - POPUP_HEIGHT) / 2.0;

        draw_rectangle(popup_x, popup_y, POPUP_WIDTH, POPUP_HEIGHT, POPUP_BG_COLOR);
        draw_rectangle_lines(
            popup_x,
            popup_y,
            POPUP_WIDTH,
            POPUP_HEIGHT,
            POPUP_BORDER_WIDTH,
            border_color,
        );

        let text_dim = measure_text(msg, None, POPUP_MSG_FONT_SIZE as u16, 1.0);
        draw_text(
            msg,
            popup_x + (POPUP_WIDTH - text_dim.width) / 2.0,
            popup_y + POPUP_MSG_Y_OFFSET,
            POPUP_MSG_FONT_SIZE,
            WHITE,
        );

        let btn_x = popup_x + (POPUP_WIDTH - POPUP_BTN_WIDTH) / 2.0;
        let btn_y = popup_y + POPUP_HEIGHT - POPUP_BTN_HEIGHT - POPUP_BTN_Y_MARGIN;
        draw_rectangle(
            btn_x,
            btn_y,
            POPUP_BTN_WIDTH,
            POPUP_BTN_HEIGHT,
            border_color,
        );

        let btn_label_dim = measure_text(POPUP_BTN_LABEL, None, POPUP_BTN_LABEL_FONT_SIZE, 1.0);
        draw_text(
            POPUP_BTN_LABEL,
            btn_x + (POPUP_BTN_WIDTH - btn_label_dim.width) / 2.0,
            btn_y + (POPUP_BTN_HEIGHT + btn_label_dim.height) / 2.0 + POPUP_BTN_LABEL_Y_OFFSET,
            POPUP_BTN_LABEL_FONT_SIZE as f32,
            WHITE,
        );

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx >= btn_x
                && mx <= btn_x + POPUP_BTN_WIDTH
                && my >= btn_y
                && my <= btn_y + POPUP_BTN_HEIGHT
            {
                return true;
            }
        }
        false
    }

    /// Checks if the game over popup should be shown and sets wrong flags.
    pub fn show_game_over_popup_if_ready(&mut self, game_over_sound: &Sound) {
        if self.state() == GameState::GameOver
            && self.mine_reveal_queue().is_empty()
            && self.particles().is_empty()
            && self.shockwaves().is_empty()
        {
            self.set_state(GameState::Lost);
            if self.sound() {
                play_sound(
                    game_over_sound,
                    PlaySoundParams {
                        looped: false,
                        volume: 0.8,
                    },
                );
            }
        }
    }

    /// Handles showing the win or game over popup and resets the game if the button is pressed.
    pub fn handle_endgame_popups(&mut self, cell_size: f32) {
        // Show win popup if player won, but only after 4 seconds
        if self.state() == GameState::Won {
            if let Some(end_time) = self.end_time() {
                if get_time() - end_time > 4.0 {
                    let msg = &format!("You Win!  Time: {:.1}s", end_time - self.start_time());
                    if self.draw_popup(cell_size, GREEN, msg) {
                        self.reset_game();
                    }
                }
            }
        }
        // Show game over popup if lost
        else if self.state() == GameState::Lost {
            let msg = "Game Over!";
            if self.draw_popup(cell_size, RED, msg) {
                self.reset_game();
            }
        }
    }
}
