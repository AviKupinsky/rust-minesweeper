//! Particle system for Minesweeper visual effects.
//!
//! This module provides a simple particle system for visual effects such as:
//! - Mine explosions (red/yellow bursts)
//! - Cell pop animations
//! - Confetti when the player wins
//!
//! It defines the `Particle` struct and utility functions to spawn, update, and draw particles.
//! All particle effects are managed as a `Vec<Particle>` in the main game state.
//!
//! Usage:
//! - Call `spawn_particles` to create explosion or pop particles at a cell.
//! - Call `spawn_confetti` to create confetti from the top of the board.
//! - Call `update_and_draw_particles` every frame to animate and render all particles.
//!
//! All constants for particle counts, speeds, and lifetimes are defined at the top for easy tweaking.

use macroquad::prelude::*;

/// Particle system constants for easy tweaking and clarity.
const MINE_PARTICLE_COUNT: usize = 24;
const NORMAL_PARTICLE_COUNT: usize = 16;
const CONFETTI_PARTICLE_COUNT: usize = 60;

const MINE_PARTICLE_SPEED_MIN: f32 = 180.0;
const MINE_PARTICLE_SPEED_RANGE: f32 = 80.0;
const NORMAL_PARTICLE_SPEED_MIN: f32 = 80.0;
const NORMAL_PARTICLE_SPEED_RANGE: f32 = 40.0;

const MINE_PARTICLE_LIFE_MIN: f32 = 0.8;
const MINE_PARTICLE_LIFE_RANGE: f32 = 0.4;
const NORMAL_PARTICLE_LIFE_MIN: f32 = 0.5;
const NORMAL_PARTICLE_LIFE_RANGE: f32 = 0.3;

const CONFETTI_SPEED_MIN: f32 = 120.0;
const CONFETTI_SPEED_MAX: f32 = 200.0;
const CONFETTI_LIFE_MIN: f32 = 2.5;
const CONFETTI_LIFE_RANGE: f32 = 1.5;
const CONFETTI_Y_MIN: f32 = -40.0;
const CONFETTI_Y_MAX: f32 = 0.0;

// Additional constants for clarity and easy tweaking
const PARTICLE_RADIUS: f32 = 4.0; // Radius of each particle
const CONFETTI_SATURATION: f32 = 0.7;
const CONFETTI_LIGHTNESS: f32 = 0.6;

/// Represents a single particle for visual effects (e.g., confetti, explosions).
#[derive(Clone, Debug)]
pub struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    color: Color,
}

impl Particle {
    /// Creates a new particle.
    pub fn new(x: f32, y: f32, vx: f32, vy: f32, life: f32, color: Color) -> Self {
        Self {
            x,
            y,
            vx,
            vy,
            life,
            color,
        }
    }

    pub fn x(&self) -> f32 {
        self.x
    }
    pub fn y(&self) -> f32 {
        self.y
    }
    pub fn vx(&self) -> f32 {
        self.vx
    }
    pub fn vy(&self) -> f32 {
        self.vy
    }
    pub fn life(&self) -> f32 {
        self.life
    }
    pub fn color(&self) -> Color {
        self.color
    }

    pub fn set_x(&mut self, x: f32) {
        self.x = x;
    }
    pub fn set_y(&mut self, y: f32) {
        self.y = y;
    }

    pub fn set_life(&mut self, life: f32) {
        self.life = life;
    }
}

/// Spawns explosion or mine particles at a given cell.
pub fn spawn_particles(
    particles: &mut Vec<Particle>,
    row: usize,
    col: usize,
    cell_size: f32,
    is_mine: bool,
    color: Option<Color>,
    top_bar_height: f32,
) {
    let x = col as f32 * cell_size + cell_size / 2.0;
    let y = row as f32 * cell_size + top_bar_height + cell_size / 2.0;
    let num_particles = if is_mine {
        MINE_PARTICLE_COUNT
    } else {
        NORMAL_PARTICLE_COUNT
    };
    let particle_color = color.unwrap_or_else(|| if is_mine { RED } else { YELLOW });
    for i in 0..num_particles {
        let angle = (i as f32 / num_particles as f32) * std::f32::consts::TAU;
        let speed = if is_mine {
            MINE_PARTICLE_SPEED_MIN + rand::gen_range(0.0, MINE_PARTICLE_SPEED_RANGE)
        } else {
            NORMAL_PARTICLE_SPEED_MIN + rand::gen_range(0.0, NORMAL_PARTICLE_SPEED_RANGE)
        };
        particles.push(Particle::new(
            x,
            y,
            speed * angle.cos(),
            speed * angle.sin(),
            if is_mine {
                MINE_PARTICLE_LIFE_MIN + rand::gen_range(0.0, MINE_PARTICLE_LIFE_RANGE)
            } else {
                NORMAL_PARTICLE_LIFE_MIN + rand::gen_range(0.0, NORMAL_PARTICLE_LIFE_RANGE)
            },
            particle_color,
        ));
    }
}

/// Spawns confetti particles from the top of the board.
pub fn spawn_confetti(particles: &mut Vec<Particle>, width: usize, cell_size: f32) {
    let width_px = width as f32 * cell_size;
    for _ in 0..CONFETTI_PARTICLE_COUNT {
        let x = rand::gen_range(0.0, width_px);
        let y = rand::gen_range(CONFETTI_Y_MIN, CONFETTI_Y_MAX);
        let speed = rand::gen_range(CONFETTI_SPEED_MIN, CONFETTI_SPEED_MAX);
        let hue = rand::gen_range(0.0, 1.0);
        let color = macroquad::color::hsl_to_rgb(hue, CONFETTI_SATURATION, CONFETTI_LIGHTNESS);
        particles.push(Particle::new(
            x,
            y,
            0.0, // Only fall straight down
            speed,
            CONFETTI_LIFE_MIN + rand::gen_range(0.0, CONFETTI_LIFE_RANGE),
            color,
        ));
    }
}

/// Updates and draws all particles. Removes dead particles.
/// Call this from your main loop.
pub fn update_and_draw_particles(particles: &mut Vec<Particle>) {
    let dt = get_frame_time(); // Time since last frame
    particles.retain_mut(|p| {
        p.set_x(p.x() + p.vx() * dt); // Update x position
        p.set_y(p.y() + p.vy() * dt); // Update y position
        p.set_life(p.life() - dt); // Decrease particle life
        if p.life() > 0.0 {
            draw_circle(p.x(), p.y(), PARTICLE_RADIUS, p.color()); // Draw particle if alive
            true // Keep particle
        } else {
            false // Remove dead particle
        }
    });
}
