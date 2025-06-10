//! Animation and effect logic for Minesweeper.
//!
//! This module contains all functions and methods related to cell animations (pop, wave/flood-fill),
//! shockwave effects, and animated mine reveals. It is responsible for visual feedback and
//! effects that enhance the gameplay experience. Board logic and UI drawing are handled in other modules.

use super::MinesweeperApp;
use crate::board::*;
use crate::gui::GameState;
use crate::particle::*;
use macroquad::audio::*;
use macroquad::prelude::*;

// --- Animation and Effect Constants ---
//
// These constants control the appearance and timing of cell animations and effects.
//
const TOP_BAR_HEIGHT: f32 = 60.0;
const POP_GROW_PHASE: f32 = 0.2; // First 20% of animation: grow
const POP_GROW_AMOUNT: f32 = 1.5; // How much to grow
const POP_SHRINK_START: f32 = 1.3; // Max scale before shrinking
const POP_LINE_WIDTH: f32 = 2.0; // Border thickness
const POP_ANIMATION_DURATION: f32 = 0.5; // Duration for pop animation

// --- Shockwave effect constants ---
const SHOCKWAVE_START_RADIUS: f32 = 30.0;
const SHOCKWAVE_GROWTH: f32 = 200.0;
const SHOCKWAVE_LINE_WIDTH: f32 = 6.0;
const SHOCKWAVE_COLOR: Color = Color::from_rgba(255, 0, 0, 180);
const REVEAL_DELAY: f32 = 0.37; // Delay between revealing mines (seconds)

impl MinesweeperApp {
    /// Handles the wave/flood-fill animation for a cell.
    /// Returns true if the animation is active and handled for this frame.
    pub fn handle_wave_animation(
        &mut self,
        row: usize,
        col: usize,
        cell_size: f32,
        win_sound: &Sound,
    ) -> bool {
        if let Some(ref mut timer) = self.wave_timers_mut()[row][col] {
            if *timer > 0.0 {
                *timer -= get_frame_time();
                return true; // Animation is still running, skip further drawing for this cell
            } else {
                self.wave_timers_mut()[row][col] = None;
                self.board_mut().uncover_cell(row, col);
                self.pop_timers_mut()[row][col] = Some(0.0);
                spawn_particles(
                    &mut self.particles_mut(),
                    row,
                    col,
                    cell_size,
                    false,
                    None,
                    TOP_BAR_HEIGHT,
                );
                self.check_win(cell_size, win_sound);
            }
        }
        false
    }

    pub fn handle_pop_animation(
        &mut self,
        row: usize,
        col: usize,
        cell: Cell,
        x: f32,
        y: f32,
        cell_size: f32,
        uncovered_color: Color,
    ) -> bool {
        if let Some(timer) = self.pop_timers()[row][col] {
            if cell != Cell::Mine {
                // Pop animation: scale up then down
                let t = (timer / POP_ANIMATION_DURATION).min(1.0);
                let scale = if t < POP_GROW_PHASE {
                    1.0 + POP_GROW_AMOUNT * t
                } else {
                    POP_SHRINK_START
                        - POP_SHRINK_START * ((t - POP_GROW_PHASE) / (1.0 - POP_GROW_PHASE))
                }
                .max(0.0);

                let cx = x + cell_size / 2.0;
                let cy = y + cell_size / 2.0;
                let size = cell_size * scale;

                draw_rectangle(
                    cx - size / 2.0,
                    cy - size / 2.0,
                    size,
                    size,
                    uncovered_color,
                );
                draw_rectangle_lines(
                    cx - size / 2.0,
                    cy - size / 2.0,
                    size,
                    size,
                    POP_LINE_WIDTH,
                    DARKGRAY,
                );

                // Draw the number if the animation is finished
                if let Cell::Number(n) = cell {
                    if t >= 1.0 {
                        self.draw_cell_number(n, cx, cy, cell_size);
                    }
                }

                // Update the pop animation timer
                self.pop_timers_mut()[row][col] = if t >= 1.0 {
                    None
                } else {
                    Some(timer + get_frame_time())
                };
                return true;
            }
        }
        false
    }

    pub fn spawn_shockwave(&mut self, row: usize, col: usize, cell_size: f32) {
        // Create a shockwave animation effect centered on the given cell.
        let x = col as f32 * cell_size + cell_size / 2.0;
        let y = row as f32 * cell_size + TOP_BAR_HEIGHT + cell_size / 2.0;
        self.shockwaves_mut().push((x, y, 0.0));
    }

    /// Updates and draws all shockwave effects. Removes finished ones.
    pub fn update_and_draw_shockwaves(&mut self) {
        self.shockwaves_mut().retain_mut(|(x, y, timer)| {
            *timer += get_frame_time();
            let radius = SHOCKWAVE_START_RADIUS + SHOCKWAVE_GROWTH * *timer;
            let alpha = (1.0 - *timer).clamp(0.0, 1.0);
            if alpha > 0.0 {
                draw_circle_lines(
                    *x,
                    *y,
                    radius,
                    SHOCKWAVE_LINE_WIDTH,
                    Color::from_rgba(
                        (SHOCKWAVE_COLOR.r * 255.0) as u8,
                        (SHOCKWAVE_COLOR.g * 255.0) as u8,
                        (SHOCKWAVE_COLOR.b * 255.0) as u8,
                        (180.0 * alpha) as u8,
                    ),
                );
                true
            } else {
                false
            }
        });
    }

    /// Reveals mines one by one with animation after game over.
    pub fn reveal_mines_with_animation(
        &mut self,
        cell_size: f32,
        mine_reveal_timer: &mut f32,
        bomb_sound: &Sound,
        mistake_sound: &Sound,
    ) {
        if self.state() == GameState::GameOver && !self.mine_reveal_queue().is_empty() {
            *mine_reveal_timer += get_frame_time();
            if *mine_reveal_timer >= REVEAL_DELAY {
                *mine_reveal_timer = 0.0;
                if let Some((r, c, is_mine)) = self.mine_reveal_queue_mut().pop() {
                    if is_mine {
                        if self.sound() {
                            play_sound(
                                bomb_sound,
                                PlaySoundParams {
                                    looped: false,
                                    volume: 0.7,
                                },
                            );
                        }
                        self.board_mut().uncover_cell(r, c);
                        spawn_particles(
                            &mut self.particles_mut(),
                            r,
                            c,
                            cell_size,
                            true,
                            None,
                            TOP_BAR_HEIGHT,
                        );
                        self.spawn_shockwave(r, c, cell_size);
                    } else {
                        if self.sound() {
                            play_sound(
                                mistake_sound,
                                PlaySoundParams {
                                    looped: false,
                                    volume: 0.7,
                                },
                            );
                        }
                        // Do NOT uncover, just mark for red X
                        self.wrong_flags_mut().push((r, c));
                    }
                }
            }
        } else {
            *mine_reveal_timer = 0.0;
        }
    }
}
