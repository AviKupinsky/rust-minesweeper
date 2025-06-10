# Experimental: SAT-based Board Difficulty Classification

I explored adding an **automatic board difficulty classification** (Easy, Medium, Hard) for the user, based on the minimum number of "guesses" required to solve a Minesweeper board using SAT logic.  
The idea was:
- **Easy:** Board can be solved with pure logic (no guessing needed).
- **Medium:** Board requires 1 or 2 guesses.
- **Hard:** Board requires more than 2 guesses.

This approach uses a recursive SAT-based solver to determine the minimum number of guesses needed.  
It works well for small boards, but is too slow for large boards, so it is not used in the final product.  
The code is preserved here for reference and review.

---


```rust
use rand_distr::WeightedIndex;
use varisat::*;
use itertools::Itertools;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardDifficulty {
    Easy,
    Medium,
    Hard,
}


 /// Places mines with bias according to the requested difficulty.
    /// Ensures the first click area is always safe.
    pub fn place_mines_weighted(
        &mut self,
        avoid_row: usize,
        avoid_col: usize,
        difficulty: BoardDifficulty,
    ) {
        // Collect all possible positions for mines, except the first click and its neighbors
        let mut positions = Vec::new();
        let mut weights = Vec::new();

        // Calculate the center of the board for distance-based weighting
        let center_row = self.height as isize / 2;
        let center_col = self.width as isize / 2;

        // Loop through every cell on the board
        for row in 0..self.height {
            for col in 0..self.width {
                // Skip the first click cell and its 8 neighbors to ensure the first click is always safe
                if (row as isize - avoid_row as isize).abs() <= 1
                    && (col as isize - avoid_col as isize).abs() <= 1
                {
                    continue;
                }
                positions.push((row, col));

                // Calculate Manhattan distance from the center for weighting
                let dist = (row as isize - center_row).abs() + (col as isize - center_col).abs();

                // Assign a weight based on the desired difficulty:
                // - Easy: farther from center = higher weight (mines more likely at edges)
                // - Medium: uniform weight (pure random)
                // - Hard: closer to center = higher weight (mines more likely in the middle)
                let weight = match difficulty {
                    BoardDifficulty::Easy => 1 + dist as usize, // farther = more likely
                    BoardDifficulty::Medium => 1,               // uniform
                    BoardDifficulty::Hard => {
                        1 + ((self.height + self.width) as isize / 2 - dist) as usize
                    } // closer = more likely
                };
                weights.push(weight);
            }
        }

        // Prepare for weighted random selection:
        // - 'rng' is our random number generator.
        // - 'chosen' keeps track of which indices (positions) we've already picked for mines,
        //    so we never place two mines in the same cell.
        // - 'dist' is a WeightedIndex, which lets us pick random indices from 'positions'
        //    with probability proportional to their weight in 'weights'.
        //    For example, if weights = [1, 2, 7], then index 0 has a 10% chance,
        //    index 1 has a 20% chance, and index 2 has a 70% chance of being picked.
        //    This is how we bias mine placement for difficulty, but every cell (except excluded ones)
        //    still has a chance to be picked.
        let mut rng = thread_rng();
        let mut chosen = HashSet::new();
        let dist = WeightedIndex::new(&weights).unwrap();

        // Clear any previous mine positions
        self.mine_positions.clear();

        // Randomly select mine positions according to the weights, until enough mines are placed
        while chosen.len() < self.mines {
            let idx = dist.sample(&mut rng); // Pick a random index, weighted by 'weights'
            if chosen.insert(idx) {
                // Only use if not already chosen
                let (row, col) = positions[idx];
                self.cells[row][col] = Cell::Mine; // Place a mine in the cell
                self.mine_positions.insert((row, col)); // Track the mine's position
            }
        }
    }


    /// Builds a CNF formula and variable map for the current board state.
    /// This encodes all number cell constraints as SAT clauses.
    /// Returns (formula, var_map) where var_map maps (row, col) -> SAT variable.
    fn to_cnf(&self) -> (CnfFormula, std::collections::HashMap<(usize, usize), Var>) {
        // Map each covered cell to a SAT variable
        let mut var_map = std::collections::HashMap::new();
        let mut var_count = 0;
        for row in 0..self.height {
            for col in 0..self.width {
                if self.cell_state(row, col) == Some(CellState::Covered) {
                    var_map.insert((row, col), Var::from_index(var_count));
                    var_count += 1;
                }
            }
        }

        // Build CNF constraints for each number cell
        let mut formula = CnfFormula::new();
        for row in 0..self.height {
            for col in 0..self.width {
                if let Some(Cell::Number(n)) = self.cell(row, col) {
                    // Collect SAT variables for all covered neighbors
                    let mut neighbor_vars = vec![];
                    for (nr, nc) in self.neighbors(row, col) {
                        if let Some(&var) = var_map.get(&(nr, nc)) {
                            neighbor_vars.push(var);
                        }
                    }

                    // Add constraint: exactly n of neighbor_vars are true (mines)
                    // This is done by encoding both "at least n" and "at most n" constraints:
                    //
                    // Why do we need both?
                    // - "At least n" alone allows for n or more mines (not exactly n).
                    // - "At most n" alone allows for n or fewer mines (not exactly n).
                    // - Using both together ensures exactly n mines among the neighbors.
                    //
                    // Example with 4 neighbors (A, B, C, D) and n = 2:
                    //   - "At least 2": For every subset of size 3, at least one must be false:
                    //       (¬A ∨ ¬B ∨ ¬C), (¬A ∨ ¬B ∨ ¬D), (¬A ∨ ¬C ∨ ¬D), (¬B ∨ ¬C ∨ ¬D)
                    //   - "At most 2": For every subset of size 3, at least one must be true:
                    //       (A ∨ B ∨ C), (A ∨ B ∨ D), (A ∨ C ∨ D), (B ∨ C ∨ D)
                    //   - Together, these force exactly 2 of A, B, C, D to be true.

                    if neighbor_vars.len() >= n as usize {
                        // At least n: for every subset of size neighbor_vars.len() - n + 1,
                        // at least one must be false (¬x1 ∨ ¬x2 ∨ ... ∨ ¬xk-n+1)
                        for subset in neighbor_vars.iter().copied().combinations(neighbor_vars.len() - n as usize + 1) {
                            let clause: Vec<_> = subset.into_iter().map(|v| Lit::from_var(v, false)).collect();
                            formula.add_clause(&clause);
                        }
                        // At most n: for every subset of size n + 1,
                        // at least one must be true (x1 ∨ x2 ∨ ... ∨ xn+1)
                        for subset in neighbor_vars.iter().copied().combinations(n as usize + 1) {
                            let clause: Vec<_> = subset.into_iter().map(|v| Lit::from_var(v, true)).collect();
                            formula.add_clause(&clause);
                        }
                    }
                }
            }
        }
        (formula, var_map)
    }


    /// Returns the minimum number of guesses needed to solve the board using SAT logic.
    /// Returns 0 if logic alone is sufficient, or the minimum number of guesses otherwise.
    /// This is a recursive, exponential-time function: use only for small boards!
    pub fn min_guesses_sat(&self) -> usize {
        // Build the CNF formula and variable map for the current board state
        let (formula, var_map) = self.to_cnf();

        // 1. Use the SAT solver to check if the board is solvable at all
        let mut solver = Solver::new();
        solver.add_formula(&formula);

        // Try to find one solution
        let sat = solver.solve().unwrap();
        if !sat {
            return usize::MAX; // No solution at all (shouldn't happen for valid boards)
        }

        // 2. For each covered cell, check if it can be both mine and not-mine in some solution
        // (If so, guessing is required)
        let mut ambiguous_cells = vec![];
        for (&(row, col), &var) in &var_map {
            // Try forcing this cell to be a mine
            let mut solver_mine = Solver::new();
            solver_mine.add_formula(&formula);
            solver_mine.add_clause(&[Lit::from_var(var, true)]);
            let mine_sat = solver_mine.solve().unwrap();

            // Try forcing this cell to be safe
            let mut solver_safe = Solver::new();
            solver_safe.add_formula(&formula);
            solver_safe.add_clause(&[Lit::from_var(var, false)]);
            let safe_sat = solver_safe.solve().unwrap();

            if mine_sat && safe_sat {
                ambiguous_cells.push((row, col));
            }
        }

        // 3. If all covered cells are determined by logic, return 0 (no guessing needed)
        if ambiguous_cells.is_empty() {
            return 0;
        }

        // 4. Otherwise, recursively try guessing each ambiguous cell as mine or safe
        let mut min_guesses = usize::MAX;
        for &(row, col) in &ambiguous_cells {
            // Guess mine
            let mut board_mine = self.clone();
            board_mine.set_cell(row, col, Cell::Mine);
            let guesses_mine = board_mine.min_guesses_sat();
            // Guess safe
            let mut board_safe = self.clone();
            // Uncovering as safe: set as Empty (or Number if you want to recalc numbers)
            board_safe.set_cell(row, col, Cell::Empty);
            let guesses_safe = board_safe.min_guesses_sat();
            let guesses = 1 + guesses_mine.min(guesses_safe);
            if guesses < min_guesses {
                min_guesses = guesses;
            }
        }
        min_guesses
    }

    /// Classifies the board's difficulty using SAT logic.
    /// - Returns "Easy" if min_guesses == 0 (logic-solvable, no guessing)
    /// - Returns "Medium" if min_guesses == 1 or 2 (some guessing)
    /// - Returns "Hard" if min_guesses > 2 (lots of guessing or ambiguous)
    pub fn classify_sat_difficulty(&self) -> BoardDifficulty {
        let min_guesses = self.min_guesses_sat();
        if min_guesses == 0 {
            BoardDifficulty::Easy
        } else if min_guesses <= 2 {
            BoardDifficulty::Medium
        } else {
            BoardDifficulty::Hard
        }
    }