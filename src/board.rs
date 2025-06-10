//! Minesweeper board logic and types.
//!
//! This module defines the core data structures and logic for the Minesweeper game board,
//! including board size presets, cell types, cell states, and all board operations such as
//! flagging, uncovering, mine placement, neighbor calculation, and flood fill reveal.
//!
//! It is the foundation for the game's state and rules, but does not handle UI or rendering.

use rand::prelude::*;
use rand::seq::SliceRandom;
use std::collections::{HashSet, VecDeque};

/// Represents the standard Minesweeper board sizes.
/// - Small: 8x8 with 10 mines (classic beginner)
/// - Medium: 16x16 with 40 mines (classic intermediate)
/// - Large: 24x24 with 99 mines (classic expert)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardSize {
    Small,
    Medium,
    Large,
}

impl BoardSize {
    /// Returns the (width, height, mines) tuple for each board size.
    pub fn params(self) -> (usize, usize, usize) {
        match self {
            BoardSize::Small => (8, 8, 10),    // Beginner
            BoardSize::Medium => (16, 16, 40), // Intermediate
            BoardSize::Large => (24, 24, 99),  // Expert
        }
    }

    /// Returns a human-readable label for each board size (for UI).
    pub fn label(self) -> &'static str {
        match self {
            BoardSize::Small => "Small",
            BoardSize::Medium => "Medium",
            BoardSize::Large => "Large",
        }
    }

    /// Returns the BoardSize variant for given width, height, and mine count.
    /// Falls back to Small if the parameters don't match a standard size.
    pub fn board_size_from_params(width: usize, height: usize, mines: usize) -> BoardSize {
        match (width, height, mines) {
            (8, 8, 10) => BoardSize::Small,
            (16, 16, 40) => BoardSize::Medium,
            (24, 24, 99) => BoardSize::Large,
            _ => BoardSize::Small, // Default/fallback
        }
    }

    /// Returns the recommended cell size (in pixels) for each board size.
    /// Used for scaling the UI and board so it fits nicely on screen.
    pub fn cell_size(self) -> f32 {
        match self {
            BoardSize::Small => 48.0,
            BoardSize::Medium => 36.0,
            BoardSize::Large => 28.0,
        }
    }
}

/// Represents a single cell on the Minesweeper board.
///
/// - `Mine`: The cell contains a mine.
/// - `Number(u8)`: The cell is adjacent to one or more mines; the number indicates how many.
/// - `Empty`: The cell is not adjacent to any mines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Mine,
    Number(u8), // Number of adjacent mines
    Empty,
}

/// Represents the state of a cell as seen by the player.
///
/// - `Covered`: The cell has not been revealed yet.
/// - `Uncovered`: The cell has been revealed.
/// - `Flagged`: The cell has been flagged by the player as potentially containing a mine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellState {
    Covered,
    Uncovered,
    Flagged,
}

/// Represents the Minesweeper game board and all its state.
///
/// Fields:
/// - `width`: The number of columns in the board.
/// - `height`: The number of rows in the board.
/// - `mines`: The total number of mines on the board.
/// - `cells`: A 2D vector representing the contents of each cell (mine, number, or empty).
/// - `states`: A 2D vector representing the state of each cell (covered, uncovered, or flagged).
/// - `mine_positions`: A set containing the coordinates of all mines on the board.
#[derive(Clone)]
pub struct Board {
    width: usize,
    height: usize,
    mines: usize,
    cells: Vec<Vec<Cell>>,
    states: Vec<Vec<CellState>>,
    mine_positions: HashSet<(usize, usize)>,
}

impl Board {
    // === Construction and Accessors ===

    /// Creates a new board with the given width, height, and mine count.
    /// All cells are initialized as `Cell::Empty` and all cell states as `CellState::Covered`.
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        let cells = vec![vec![Cell::Empty; width]; height];
        let states = vec![vec![CellState::Covered; width]; height];
        Board {
            width,
            height,
            mines,
            cells,
            states,
            mine_positions: HashSet::new(),
        }
    }

    /// Returns the width of the board.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the board.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns the number of mines on the board.
    pub fn mines(&self) -> usize {
        self.mines
    }

    // === Cell and State Access ===

    /// Returns the cell at the given position, if valid.
    ///
    /// Note: Returns an owned value (`Option<Cell>`) using `.copied()`.
    pub fn cell(&self, row: usize, col: usize) -> Option<Cell> {
        self.cells.get(row).and_then(|r| r.get(col)).copied()
    }

    /// Returns the state of the cell at the given position, if valid.
    ///
    /// Note: Returns an owned value (`Option<CellState>`) using `.copied()`.
    pub fn cell_state(&self, row: usize, col: usize) -> Option<CellState> {
        self.states.get(row).and_then(|r| r.get(col)).copied()
    }

    // === Cell Manipulation ===

    /// Flags the cell at the given position, if valid.
    pub fn flag_cell(&mut self, row: usize, col: usize) {
        if let Some(state) = self.states.get_mut(row).and_then(|r| r.get_mut(col)) {
            *state = CellState::Flagged;
        }
    }

    /// Unflags the cell at the given position, if valid.
    pub fn unflag_cell(&mut self, row: usize, col: usize) {
        if let Some(state) = self.states.get_mut(row).and_then(|r| r.get_mut(col)) {
            if *state == CellState::Flagged {
                *state = CellState::Covered;
            }
        }
    }

    /// Uncovers the cell at the given position, if valid.
    pub fn uncover_cell(&mut self, row: usize, col: usize) {
        if let Some(state) = self.states.get_mut(row).and_then(|r| r.get_mut(col)) {
            *state = CellState::Uncovered;
        }
    }

    // === Mine Logic ===

    /// Returns a reference to the set of all mine positions.
    pub fn mine_positions(&self) -> &HashSet<(usize, usize)> {
        &self.mine_positions
    }

    /// Randomly places mines, avoiding the given cell and its neighbors.
    pub fn place_mines_avoiding(&mut self, avoid_row: usize, avoid_col: usize) {
        // Build a list of all positions except the avoid cell and its neighbors
        let mut positions = Vec::new();
        for row in 0..self.height {
            for col in 0..self.width {
                // Avoid the clicked cell and its neighbors
                if (row as isize - avoid_row as isize).abs() <= 1
                    && (col as isize - avoid_col as isize).abs() <= 1
                {
                    continue;
                }
                positions.push((row, col));
            }
        }
        let mut rng = thread_rng();
        positions.shuffle(&mut rng);

        self.mine_positions.clear();
        for &(row, col) in positions.iter().take(self.mines) {
            self.cells[row][col] = Cell::Mine;
            self.mine_positions.insert((row, col));
        }
    }

    // === Neighbor and Number Logic ===

    /// Returns an iterator over all valid neighbor coordinates for a given cell.
    /// This helper avoids code duplication in neighbor logic.
    pub fn neighbors(&self, row: usize, col: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
        (-1..=1).flat_map(move |dr| {
            (-1..=1).filter_map(move |dc| {
                if dr == 0 && dc == 0 {
                    None
                } else {
                    let nr = row as isize + dr;
                    let nc = col as isize + dc;
                    if nr >= 0 && nr < self.height as isize && nc >= 0 && nc < self.width as isize {
                        Some((nr as usize, nc as usize))
                    } else {
                        None
                    }
                }
            })
        })
    }

    /// Calculates numbers for each cell based on adjacent mines.
    pub fn calculate_numbers(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                if let Cell::Mine = self.cells[row][col] {
                    continue;
                }
                let count = self
                    .neighbors(row, col)
                    .filter(|&(nr, nc)| self.cells[nr][nc] == Cell::Mine)
                    .count();
                self.cells[row][col] = if count == 0 {
                    Cell::Empty
                } else {
                    Cell::Number(count as u8)
                };
            }
        }
    }

    // === Flood Fill (Reveal) Logic ===

    /// Reveals all connected empty cells and their neighbors (flood fill), and returns their positions and wave distance.
    /// Each tuple is (row, col, distance_from_origin).
    /// This is a classic BFS flood fill, revealing all connected empty cells and their neighbors,
    /// and tracking the "wave" distance from the starting cell.
    pub fn flood_fill_wave(&mut self, row: usize, col: usize) -> Vec<(usize, usize, usize)> {
        let mut queue = VecDeque::new();
        let mut revealed = Vec::new();
        let mut visited = vec![vec![false; self.width]; self.height];

        queue.push_back((row, col, 0));
        visited[row][col] = true;

        while let Some((r, c, dist)) = queue.pop_front() {
            if self.states[r][c] == CellState::Uncovered {
                continue;
            }
            self.states[r][c] = CellState::Uncovered;
            revealed.push((r, c, dist));
            if self.cells[r][c] == Cell::Empty {
                for (nr, nc) in self.neighbors(r, c) {
                    if !visited[nr][nc] && self.states[nr][nc] == CellState::Covered {
                        queue.push_back((nr, nc, dist + 1));
                        visited[nr][nc] = true;
                    }
                }
            }
        }
        revealed
    }

    // === Testing Helpers ===

    /// Allows tests to set a cell value directly.
    pub fn set_cell(&mut self, row: usize, col: usize, cell: Cell) {
        if let Some(r) = self.cells.get_mut(row) {
            if let Some(c) = r.get_mut(col) {
                *c = cell;
            }
        }
    }

    /// Allows tests to set a cell state directly.
    pub fn set_cell_state(&mut self, row: usize, col: usize, state: CellState) {
        if let Some(r) = self.states.get_mut(row) {
            if let Some(s) = r.get_mut(col) {
                *s = state;
            }
        }
    }

    /// Allows tests to insert a mine position directly (for testing only).
    pub fn insert_mine_position(&mut self, row: usize, col: usize) {
        self.mine_positions.insert((row, col));
    }

    /// Allows tests to check if mine_positions is empty (for testing only).
    pub fn mine_positions_is_empty(&self) -> bool {
        self.mine_positions.is_empty()
    }
}
