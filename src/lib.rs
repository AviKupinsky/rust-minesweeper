pub mod board;                // Exposes the board module to others
pub use board::*; // Re-exports for easy access
pub use gui::MinesweeperApp;            // Re-export main app struct
pub use gui::GameState;
pub use particle::Particle;
mod gui;                      // Keeps gui private, but you re-export types below
mod particle;             // Exposes particle module
mod gui_animation;        // Exposes animation helpers
mod gui_board;            // Exposes GUI board helpers
mod gui_popup;            // Exposes popup helpers
mod gui_ui;               // Exposes UI helpers
