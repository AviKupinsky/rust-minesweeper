//! UI drawing and input logic for Minesweeper.
//!
//! This module contains all functions and methods related to drawing the top bar UI,
//! including the flags left counter, timer, clock icon, and "New Game" button.
//! It also handles user input for the "New Game" button. Board logic and animation
//! are handled in other modules.

use super::MinesweeperApp;
use crate::board::*;
use crate::gui::GameState;
use macroquad::prelude::*;

// === UI Layout and Style Constants ===
const TOP_BAR_HEIGHT: f32 = 60.0;
const ICON_SIZE: f32 = 32.0;
const BTN_W: f32 = 70.0;
const BTN_H: f32 = 36.0;
const FONT_SIZE: f32 = 20.0;
const ICON_Y: f32 = 18.0;
const ICON_TEXT_OFFSET: f32 = 0.8; // Multiplier for icon size to position text vertically
const BTN_LABEL_SUFFIX: &str = " v";

// Colors
const COLOR_TOP_BAR: Color = Color::from_rgba(255, 140, 0, 255);
const COLOR_BTN: Color = Color::from_rgba(255, 220, 120, 255);
const COLOR_BTN_SELECTED: Color = Color::from_rgba(255, 220, 120, 255);
const COLOR_BTN_UNSELECTED: Color = Color::from_rgba(220, 220, 220, 255);
const COLOR_DROPDOWN_BG: Color = Color::from_rgba(245, 245, 245, 255);
const COLOR_TEXT: Color = BLACK;

impl MinesweeperApp {
    /// Returns dynamic spacing for top bar elements based on board size.
    pub fn top_bar_spacing(&self) -> f32 {
        match self.board_size() {
            BoardSize::Small => 20.0,
            BoardSize::Medium => 48.0,
            BoardSize::Large => 64.0,
        }
    }

    /// Draws the entire top bar, calling helper functions for each section.
    /// Note: The dropdown menu itself should be drawn after the board for proper layering!
    pub fn draw_top_bar(
        &mut self,
        cell_size: f32,
        flag_texture: &Texture2D,
        clock_texture: &Texture2D,
        new_game_texture: &Texture2D,
        mute_texture: &Texture2D,      // <-- Add this
        volume_texture: &Texture2D,    // <-- Add this
    ) {
        // Draw the background of the top bar
        let bar_width = self.board().width() as f32 * cell_size;
        draw_rectangle(0.0, 0.0, bar_width, TOP_BAR_HEIGHT, COLOR_TOP_BAR);

        let mut x = self.top_bar_start_x();
        let spacing = self.top_bar_spacing();

        // Draw flags left section and update x
        x = self.draw_flags_left_section(x, flag_texture, spacing);

        // Draw timer section and update x
        x = self.draw_timer_section(x, clock_texture, spacing);

        // Draw board size dropdown button (but NOT the dropdown menu itself)
        x = self.draw_board_size_dropdown_button(x, spacing);

        // Draw new game icon and update x
        x = self.draw_new_game_icon(x, new_game_texture, spacing);

        // Draw sound icon (future)
        self.draw_sound_icon(x, volume_texture,mute_texture);
    }

    /// Returns the recommended starting X position for the top bar,
    /// based on the board width and cell size.
    pub fn top_bar_start_x(&self) -> f32 {
        let bar_width = self.board().width() as f32 * self.cell_size();
        (bar_width * 0.08).max(12.0)
    }

    /// Draws the flag icon and flags left counter.
    /// Returns the new x position after this section.
    pub fn draw_flags_left_section(
        &self,
        mut x: f32,
        flag_texture: &Texture2D,
        spacing: f32,
    ) -> f32 {
        draw_texture_ex(
            flag_texture,
            x,
            ICON_Y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(ICON_SIZE, ICON_SIZE)),
                ..Default::default()
            },
        );
        x += ICON_SIZE + 4.0;
        let flags_placed = (0..self.board().height())
            .flat_map(|row| (0..self.board().width()).map(move |col| (row, col)))
            .filter(|&(row, col)| self.board().cell_state(row, col) == Some(CellState::Flagged))
            .count();
        let flags_left = self.board().mines() as isize - flags_placed as isize;
        draw_text(
            &flags_left.to_string(),
            x,
            ICON_Y + ICON_SIZE * ICON_TEXT_OFFSET,
            FONT_SIZE,
            COLOR_TEXT,
        );
        x + measure_text(&flags_left.to_string(), None, FONT_SIZE as u16, 1.0).width + spacing
    }

    /// Draws the clock icon and timer.
    /// Returns the new x position after this section.
    pub fn draw_timer_section(&self, mut x: f32, clock_texture: &Texture2D, spacing: f32) -> f32 {
        draw_texture_ex(
            clock_texture,
            x,
            ICON_Y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(ICON_SIZE, ICON_SIZE)),
                ..Default::default()
            },
        );
        x += ICON_SIZE + 4.0;
        let elapsed_time = if let Some(end_time) = self.end_time() {
            end_time - self.start_time()
        } else if self.state() == GameState::Running {
            get_time() - self.start_time()
        } else {
            0.0
        };
        let total_seconds = elapsed_time as u32;
        let time_str = format!("{:02}:{:02}", total_seconds / 60, total_seconds % 60);
        draw_text(
            &time_str,
            x,
            ICON_Y + ICON_SIZE * ICON_TEXT_OFFSET,
            FONT_SIZE,
            COLOR_TEXT,
        );
        x + measure_text(&time_str, None, FONT_SIZE as u16, 1.0).width + spacing
    }

    /// Draws the board size dropdown button (but NOT the dropdown menu itself).
    /// Returns the new x position after this section.
    fn draw_board_size_dropdown_button(&mut self, x: f32, spacing: f32) -> f32 {
        let btn_label = format!("{}{}", self.board_size().label(), BTN_LABEL_SUFFIX);
        draw_rectangle(x, ICON_Y, BTN_W, BTN_H, COLOR_BTN);
        let label_dim = measure_text(&btn_label, None, FONT_SIZE as u16, 1.0);
        draw_text(
            &btn_label,
            x + (BTN_W - label_dim.width) / 2.0,
            ICON_Y + BTN_H * 0.7,
            FONT_SIZE,
            COLOR_TEXT,
        );
        // Handle dropdown click
        if is_mouse_button_pressed(MouseButton::Left) {
                if self.ignore_next_size_popup_click() {
                    self.set_ignore_next_size_popup_click(false); // Reset the flag
                } else {
                    let (mx, my) = mouse_position();
                    if mx >= x && mx <= x + BTN_W && my >= ICON_Y && my <= ICON_Y + BTN_H {
                        self.set_show_size_popup(true);
                    }
                }
        }
        x + BTN_W + spacing
    }

    /// Draws the dropdown menu for board size selection.
    /// Call this AFTER drawing the board, so it appears on top of the cells.
    pub fn draw_board_size_dropdown_menu(&mut self, x: f32) {
        if !self.show_size_popup() || self.ignore_next_size_popup_click(){
            return;
        }
        let sizes = [BoardSize::Small, BoardSize::Medium, BoardSize::Large];
        let popup_x = x;
        let popup_y = ICON_Y + BTN_H;
        let popup_w = BTN_W;
        let popup_h = sizes.len() as f32 * BTN_H;
        draw_rectangle(popup_x, popup_y, popup_w, popup_h, COLOR_DROPDOWN_BG);
        for (i, &size) in sizes.iter().enumerate() {
            let by = popup_y + i as f32 * BTN_H;
            draw_rectangle(
                popup_x,
                by,
                popup_w,
                BTN_H,
                if self.board_size() == size {
                    COLOR_BTN_SELECTED
                } else {
                    COLOR_BTN_UNSELECTED
                },
            );
            let label = size.label();
            let label_dim = measure_text(label, None, FONT_SIZE as u16, 1.0);
            draw_text(
                label,
                popup_x + (popup_w - label_dim.width) / 2.0,
                by + BTN_H * 0.7,
                FONT_SIZE,
                COLOR_TEXT,
            );
            // Handle click on a size option
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mx, my) = mouse_position();
                if mx >= popup_x && mx <= popup_x + popup_w && my >= by && my <= by + BTN_H {
                    if self.board_size() == size {
                        return;
                    }
                    self.set_board_size(size);
                    let (w, h, _) = size.params();
                    use macroquad::window::request_new_screen_size;
                    request_new_screen_size(
                        w as f32 * size.cell_size(),
                        h as f32 * size.cell_size() + TOP_BAR_HEIGHT,
                    );
                    self.reset_game();
                    // self.set_show_size_popup(false); // Close the dropdown
                    self.set_ignore_next_size_popup_click(true);  // Ignore the next click to prevent immediate reopen
                    return;
                }
            }
        }
        // Optional: click outside to close the popup
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if !(mx >= popup_x
                && mx <= popup_x + popup_w
                && my >= popup_y
                && my <= popup_y + popup_h)
                && !(mx >= x && mx <= x + BTN_W && my >= ICON_Y && my <= ICON_Y + BTN_H)
            {
                self.set_show_size_popup(false);
            }
        }
    }

    /// Draws the new game icon and handles click.
    /// Returns the new x position after this section.
    fn draw_new_game_icon(&mut self, x: f32, new_game_texture: &Texture2D, spacing: f32) -> f32 {
        draw_texture_ex(
            new_game_texture,
            x,
            ICON_Y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(ICON_SIZE, ICON_SIZE)),
                ..Default::default()
            },
        );
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx >= x && mx <= x + ICON_SIZE && my >= ICON_Y && my <= ICON_Y + ICON_SIZE {
                self.reset_game();
            }
        }
        x + ICON_SIZE + spacing
    }

    /// Draws the sound icon (future).
    fn draw_sound_icon(&mut self, x: f32, sound_texture: &Texture2D, mute_texture: &Texture2D, ) {
        let sound_icon = if self.sound() {
            sound_texture // Show muted icon
        } else {
            mute_texture// Show volume icon
        };
        draw_texture_ex(
            sound_icon,
            x,
            ICON_Y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(ICON_SIZE, ICON_SIZE)),
                ..Default::default()
            },
        );
        let (mx, my) = mouse_position();
        if is_mouse_button_pressed(MouseButton::Left)
            && mx >= x && mx <= x + ICON_SIZE
            && my >= ICON_Y && my <= ICON_Y + ICON_SIZE
        {
            self.set_sound(!self.sound());
        }
    }

    /// Draws the dropdown menu for board size selection at the correct position.
    /// This should be called after drawing the board, so it appears on top.
    pub fn draw_top_bar_dropdown_menu(
        &mut self,
        flag_texture: &Texture2D,
        clock_texture: &Texture2D,
    ) {
        let mut x = self.top_bar_start_x();
        let spacing = self.top_bar_spacing();
        x = self.draw_flags_left_section(x, flag_texture, spacing);
        x = self.draw_timer_section(x, clock_texture, spacing);
        self.draw_board_size_dropdown_menu(x);
    }
}
