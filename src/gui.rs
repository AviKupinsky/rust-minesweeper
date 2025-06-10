//! Main GUI entry point and application loop for Minesweeper.
//!
//! This module defines the `MinesweeperApp` struct, the main game state, and the core game loop.
//! It is responsible for initializing the board, handling the main event loop, and orchestrating
//! calls to the board logic, UI, animation, and popup modules. Most helper functions and logic
//! are delegated to submodules for clarity and maintainability.
//!
//! Key responsibilities:
//! - Holds the main game state and board
//! - Loads assets (textures)
//! - Runs the main async game loop (`run`)
//! - Delegates drawing, input, and animation to submodules
//! - Handles game reset and state transitions

use crate::board::*;
use crate::particle::*;
use macroquad::audio::*;
use macroquad::prelude::*;

// --- Asset file paths ---
const FLAG_TEXTURE_PATH: &str = "assets/flag.png"; // Flag icon
const MINE_TEXTURE_PATH: &str = "assets/blast.png"; // Mine icon
const CLOCK_TEXTURE_PATH: &str = "assets/clock.png"; // Clock icon
const MUTE_TEXTURE_PATH: &str = "assets/mute.png"; // Mute/sound icon
const SYNCHRONIZE_TEXTURE_PATH: &str = "assets/synchronize.png"; // New game/restart icon
const VOLUME_TEXTURE_PATH: &str = "assets/volume.png"; // Volume/sound-on icon
const FLAG_SOUND_PATH: &str = "assets/flag.wav";
const BOMB_SOUND_PATH: &str = "assets/bomb.wav";
const REMOVE_FLAG_SOUND_PATH: &str = "assets/remove_flag.wav";
const FLIP_SOUND_PATH: &str = "assets/flip.wav";
const WAVE_SOUND_PATH: &str = "assets/wave.wav";
const MISTAKE_SOUND_PATH: &str = "assets/mistake.wav";
const GAME_OVER_SOUND_PATH: &str = "assets/game_over.wav";
const WIN_SOUND_PATH: &str = "assets/win.wav";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Represents the current state of the game.
/// Used to control input, animation, and UI transitions.
pub enum GameState {
    NotStarted, // Before first click
    Running,    // Game in progress
    GameOver,   // Game is over, animation running
    Won,        // Game is won (optional, for win popup)
    Lost,       // Game is lost (for loss popup)
}

/// The main application struct for the Minesweeper game.
/// Holds the board, game state, and all UI/animation state.
pub struct MinesweeperApp {
    // --- Board and game state ---
    board: Board, // The game board state and mine locations

    // --- Booleans (game state flags) ---
    state: GameState, // The current game state

    // --- Board size selection state ---
    board_size: BoardSize, // Current selected board size (Small, Medium, Large)
    show_size_popup: bool, // Whether the board size dropdown is visible
    ignore_next_size_popup_click: bool, // Flag to ignore the next click (prevents dropdown reopening)
    cell_size: f32,                     // Size of each cell in pixels

    sound: bool, // Whether sound is muted

    // --- Timers and time tracking ---
    start_time: f64,       // Time when the game started (seconds since epoch)
    end_time: Option<f64>, // Time when the player won (if any)

    // --- Animation and effect state ---
    pop_timers: Vec<Vec<Option<f32>>>, // 2D array of timers for pop animations for each cell
    wave_timers: Vec<Vec<Option<f32>>>, // 2D array of timers for wave/flood-fill animations
    particles: Vec<Particle>, // List of all active particle effects (confetti, explosions, etc.)
    shockwaves: Vec<(f32, f32, f32)>, // List of active shockwave effects (x, y, timer)

    // --- Reveal and flag state ---
    mine_reveal_queue: Vec<(usize, usize, bool)>, // Queue of mines to reveal (for animated mine reveal)
    wrong_flags: Vec<(usize, usize)>, // List of wrongly flagged cells (for highlighting mistakes)
}

impl MinesweeperApp {
    /// Returns a reference to the board (read-only).
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Returns a mutable reference to the board (for modification).
    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    /// Returns the current game state (read-only).
    pub fn state(&self) -> GameState {
        self.state
    }

    /// Sets the current game state.
    pub fn set_state(&mut self, state: GameState) {
        self.state = state;
    }

    /// Returns the start time (read-only).
    pub fn start_time(&self) -> f64 {
        self.start_time
    }

    /// Sets the start time.
    pub fn set_start_time(&mut self, time: f64) {
        self.start_time = time;
    }

    /// Returns the end time (read-only).
    pub fn end_time(&self) -> Option<f64> {
        self.end_time
    }

    /// Sets the end time.
    pub fn set_end_time(&mut self, time: Option<f64>) {
        self.end_time = time;
    }

    /// Returns a reference to the pop_timers (read-only).
    pub fn pop_timers(&self) -> &Vec<Vec<Option<f32>>> {
        &self.pop_timers
    }

    /// Returns a mutable reference to the pop_timers (for modification).
    pub fn pop_timers_mut(&mut self) -> &mut Vec<Vec<Option<f32>>> {
        &mut self.pop_timers
    }

    /// Returns a mutable reference to the wave_timers (for modification).
    pub fn wave_timers_mut(&mut self) -> &mut Vec<Vec<Option<f32>>> {
        &mut self.wave_timers
    }

    /// Returns a reference to the particles (read-only).
    pub fn particles(&self) -> &Vec<Particle> {
        &self.particles
    }

    /// Returns a mutable reference to the particles (for modification).
    pub fn particles_mut(&mut self) -> &mut Vec<Particle> {
        &mut self.particles
    }

    /// Returns a reference to the shockwaves (read-only).
    pub fn shockwaves(&self) -> &Vec<(f32, f32, f32)> {
        &self.shockwaves
    }

    /// Returns a mutable reference to the shockwaves (for modification).
    pub fn shockwaves_mut(&mut self) -> &mut Vec<(f32, f32, f32)> {
        &mut self.shockwaves
    }

    /// Returns a reference to the mine_reveal_queue (read-only).
    pub fn mine_reveal_queue(&self) -> &Vec<(usize, usize, bool)> {
        &self.mine_reveal_queue
    }

    /// Returns a mutable reference to the mine_reveal_queue (for modification).
    pub fn mine_reveal_queue_mut(&mut self) -> &mut Vec<(usize, usize, bool)> {
        &mut self.mine_reveal_queue
    }

    /// Returns a reference to the wrongly flagged cells (read-only).
    pub fn wrong_flags(&self) -> &Vec<(usize, usize)> {
        &self.wrong_flags
    }

    /// Returns a mutable reference to the wrongly flagged cells (for modification).
    pub fn wrong_flags_mut(&mut self) -> &mut Vec<(usize, usize)> {
        &mut self.wrong_flags
    }

    /// Returns the current cell size (read-only).
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Returns the current board size.
    pub fn board_size(&self) -> BoardSize {
        self.board_size
    }

    /// Sets the board size.
    pub fn set_board_size(&mut self, size: BoardSize) {
        self.board_size = size;
    }

    /// Returns whether the size popup is shown.
    pub fn show_size_popup(&self) -> bool {
        self.show_size_popup
    }

    /// Sets whether the size popup is shown.
    pub fn set_show_size_popup(&mut self, show: bool) {
        self.show_size_popup = show;
    }

    /// Returns whether the next size popup click should be ignored.
    pub fn ignore_next_size_popup_click(&self) -> bool {
        self.ignore_next_size_popup_click
    }

    /// Sets whether the next size popup click should be ignored.
    pub fn set_ignore_next_size_popup_click(&mut self, value: bool) {
        self.ignore_next_size_popup_click = value;
    }

    /// Returns whether sound is muted.
    pub fn sound(&self) -> bool {
        self.sound
    }

    /// Sets whether sound is muted.
    pub fn set_sound(&mut self, value: bool) {
        self.sound = value;
    }

    /// Helper function to create a new MinesweeperApp with all fields initialized.
    /// Used by both `new` and `reset_game` to avoid code duplication.
    fn make_empty(
        width: usize,
        height: usize,
        mines: usize,
        show_size_popup: bool,
        sound: bool,
    ) -> Self {
        Self {
            // --- Board and game state ---
            board: Board::new(width, height, mines),

            // --- Board size selection state ---
            board_size: BoardSize::board_size_from_params(width, height, mines),
            show_size_popup: show_size_popup,
            ignore_next_size_popup_click: false,

            cell_size: BoardSize::board_size_from_params(width, height, mines).cell_size(),
            sound: sound, // Whether sound is muted

            // --- Booleans (game state flags) ---
            // --- Game state ---
            state: GameState::NotStarted,

            // --- Timers and time tracking ---
            start_time: 0.0,
            end_time: None,

            // --- Animation and effect state ---
            pop_timers: vec![vec![None; width]; height],
            wave_timers: vec![vec![None; width]; height],
            particles: Vec::new(),
            shockwaves: Vec::new(),

            // --- Reveal and flag state ---
            mine_reveal_queue: Vec::new(),
            wrong_flags: Vec::new(),
        }
    }

    /// Creates a new MinesweeperApp instance with the given board size and mine count.
    /// This is the main constructor, called at program start.
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        Self::make_empty(width, height, mines, false, true)
    }

    /// Resets the current game to its initial state, keeping the same board size and mine count.
    /// Called when the player clicks "New Game" or restarts.
    pub fn reset_game(&mut self) {
        let (width, height, mines) = self.board_size.params();
        *self = Self::make_empty(width, height, mines, self.show_size_popup, self.sound);
    }

    /// Main game loop. Handles drawing, input, and game logic.
    /// This version is broken into smaller helper functions for clarity.
    pub async fn run(&mut self) {
        // Load textures and audio using constants for file paths and audio  paths
        let flag_texture = load_texture(FLAG_TEXTURE_PATH).await.unwrap();
        let mine_texture = load_texture(MINE_TEXTURE_PATH).await.unwrap();
        let clock_texture = load_texture(CLOCK_TEXTURE_PATH).await.unwrap();
        let mute_texture = load_texture(MUTE_TEXTURE_PATH).await.unwrap(); // Mute/sound icon
        let synchronize_texture = load_texture(SYNCHRONIZE_TEXTURE_PATH).await.unwrap(); // New game/restart icon
        let volume_texture = load_texture(VOLUME_TEXTURE_PATH).await.unwrap();
        let flag_sound: Sound = load_sound(FLAG_SOUND_PATH).await.unwrap();
        let bomb_sound: Sound = load_sound(BOMB_SOUND_PATH).await.unwrap();
        let remove_flag_sound: Sound = load_sound(REMOVE_FLAG_SOUND_PATH).await.unwrap();
        let flip_sound: Sound = load_sound(FLIP_SOUND_PATH).await.unwrap();
        let wave_sound: Sound = load_sound(WAVE_SOUND_PATH).await.unwrap();
        let mistake_sound: Sound = load_sound(MISTAKE_SOUND_PATH).await.unwrap();
        let game_over_sound: Sound = load_sound(GAME_OVER_SOUND_PATH).await.unwrap();
        let win_sound: Sound = load_sound(WIN_SOUND_PATH).await.unwrap();

        let mut mine_reveal_timer = 0.0;

        loop {
            // 1. Clear the screen to a light gray background
            clear_background(LIGHTGRAY);

            // 2. Draw the top bar UI (flags, timer, new game button, sound)
            self.draw_top_bar(
                self.cell_size,
                &flag_texture,
                &clock_texture,
                &synchronize_texture,
                &mute_texture,
                &volume_texture,
            );

            // 3. Draw the Minesweeper board (cells)
            self.draw_board(self.cell_size, &flag_texture, &mine_texture, &win_sound);

            // 4. Draw the dropdown menu LAST, so it appears on top of the cells
            if self.show_size_popup {
                self.draw_top_bar_dropdown_menu(&flag_texture, &clock_texture);
            }

            // 5. Update and draw all particle effects (confetti, explosions, etc.)
            update_and_draw_particles(&mut self.particles);

            // 6. Update and draw all shockwave effects
            self.update_and_draw_shockwaves();

            // 7. Reveal mines with animation
            self.reveal_mines_with_animation(
                self.cell_size,
                &mut mine_reveal_timer,
                &bomb_sound,
                &mistake_sound,
            );

            // 8. Show game over popup if ready (after all animations)
            self.show_game_over_popup_if_ready(&game_over_sound);

            // 9. Handle left mouse click (main game logic)
            if !self.show_size_popup {
                if is_mouse_button_pressed(MouseButton::Left)
                    && (self.state == GameState::NotStarted || self.state == GameState::Running)
                {
                    if let Some((row, col)) = self.mouse_to_cell(self.cell_size) {
                        if self.board.cell_state(row, col) == Some(CellState::Covered) {
                            self.handle_left_click(
                                row,
                                col,
                                self.cell_size,
                                &mut mine_reveal_timer,
                                &bomb_sound,
                                &flip_sound,
                                &wave_sound,
                                &win_sound,
                            );
                        }
                    }
                }

                // 10. Handle right mouse click (flag/unflag)
                if is_mouse_button_pressed(MouseButton::Right) && self.state == GameState::Running {
                    if let Some((row, col)) = self.mouse_to_cell(self.cell_size) {
                        self.handle_right_click(row, col, &flag_sound, &remove_flag_sound);
                    }
                }
            }

            // 11. Handle endgame popups (win/game over)
            self.handle_endgame_popups(self.cell_size);

            // 12. Wait for the next frame (yields to the event loop)
            next_frame().await;
        }
    }
}
