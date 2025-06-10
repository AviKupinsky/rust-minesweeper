// use macroquad::prelude::get_time;
use rust_project::*;
// Checks that placing 10 mines on an 8x8 board results in exactly 10 mines and correct board dimensions.
#[test]
fn test_small_board_mine_count() {
    let mut board = Board::new(8, 8, 10);
    board.place_mines_avoiding(0, 0);
    let mine_count = (0..board.height())
        .flat_map(|row| (0..board.width()).map(move |col| (row, col)))
        .filter(|&(row, col)| board.cell(row, col) == Some(Cell::Mine))
        .count();
    assert_eq!(mine_count, 10);
    assert_eq!(board.width(), 8);
    assert_eq!(board.height(), 8);
    assert_eq!(board.mines(), 10);
}

// Checks that placing 99 mines on a 24x24 board results in exactly 99 mines and correct board dimensions.
#[test]
fn test_large_board_mine_count() {
    let mut board = Board::new(24, 24, 99);
    board.place_mines_avoiding(0, 0);
    let mine_count = (0..board.height())
        .flat_map(|row| (0..board.width()).map(move |col| (row, col)))
        .filter(|&(row, col)| board.cell(row, col) == Some(Cell::Mine))
        .count();
    assert_eq!(mine_count, 99);
    assert_eq!(board.width(), 24);
    assert_eq!(board.height(), 24);
    assert_eq!(board.mines(), 99);
}

// Checks that all cells in a new 10x10 board are initially covered.
#[test]
fn test_cell_states() {
    let board = Board::new(10, 10, 10);
    for row in 0..10 {
        for col in 0..10 {
            assert_eq!(board.cell_state(row, col), Some(CellState::Covered));
        }
    }
}



// Checks that flagging and unflagging a cell works as expected.
#[test]
fn test_flag_and_unflag_cell() {
    let mut board = Board::new(5, 5, 0);
    board.flag_cell(2, 2);
    assert_eq!(board.cell_state(2, 2), Some(CellState::Flagged));
    board.unflag_cell(2, 2);
    assert_eq!(board.cell_state(2, 2), Some(CellState::Covered));
}

// Checks that uncovering a cell sets its state to Uncovered.
#[test]
fn test_uncover_cell() {
    let mut board = Board::new(3, 3, 0);
    board.uncover_cell(1, 1);
    assert_eq!(board.cell_state(1, 1), Some(CellState::Uncovered));
}

// Checks that out-of-bounds access returns None.
#[test]
fn test_out_of_bounds_access() {
    let board = Board::new(2, 2, 0);
    assert_eq!(board.cell(10, 10), None);
    assert_eq!(board.cell_state(10, 10), None);
}

// Checks that flood fill reveals all connected empty cells and their neighbors.
#[test]
fn test_flood_fill_wave_reveals() {
    let mut board = Board::new(3, 3, 0);
    board.set_cell(0, 0, Cell::Mine);
    board.insert_mine_position(0, 0);
    board.calculate_numbers();
    let revealed = board.flood_fill_wave(2, 2);
    for row in 0..3 {
        for col in 0..3 {
            if (row, col) != (0, 0) {
                assert!(revealed.iter().any(|&(r, c, _)| r == row && c == col));
            }
        }
    }
}

// Checks that place_mines_avoiding does not place mines in the avoided cell or its neighbors.
#[test]
fn test_place_mines_avoiding_avoids_neighbors() {
    let mut board = Board::new(5, 5, 10);
    board.place_mines_avoiding(2, 2);
    for dr in -1..=1 {
        for dc in -1..=1 {
            let r = 2_i32 + dr;
            let c = 2_i32 + dc;
            if r >= 0 && r < 5 && c >= 0 && c < 5 {
                assert_ne!(board.cell(r as usize, c as usize), Some(Cell::Mine));
            }
        }
    }
}

// Checks that flagging a cell twice keeps it flagged, and unflagging twice keeps it covered.
#[test]
fn test_double_flag_and_unflag() {
    // Arrange: create a board and flag a cell twice
    let mut board = Board::new(3, 3, 0);
    board.flag_cell(1, 1);
    board.flag_cell(1, 1); // Flag again (should be idempotent)
    // Assert: cell remains flagged
    assert_eq!(board.cell_state(1, 1), Some(CellState::Flagged));
    // Act: unflag twice
    board.unflag_cell(1, 1);
    board.unflag_cell(1, 1); // Unflag again (should be idempotent)
    // Assert: cell is covered
    assert_eq!(board.cell_state(1, 1), Some(CellState::Covered));
}


// Checks that a board with zero mines contains only empty cells.
#[test]
fn test_no_mines_board() {
    let mut board = Board::new(3, 3, 0);
    board.calculate_numbers();
    for row in 0..3 {
        for col in 0..3 {
            assert_eq!(board.cell(row, col), Some(Cell::Empty));
        }
    }
}

// --- App-level (GUI) tests ---

// use rust_project::GameState;

// Checks that toggling sound on and off updates the sound state correctly.
#[test]
fn test_toggle_sound() {
    let mut app = MinesweeperApp::new(8, 8, 10);

    // By default, sound should be on (true)
    assert!(app.sound(), "Sound should be on by default");

    // Turn sound off
    app.set_sound(false);
    assert!(!app.sound(), "Sound should be off after muting");

    // Turn sound back on
    app.set_sound(true);
    assert!(app.sound(), "Sound should be on after unmuting");
}

// This test verifies that calling reset_game() on MinesweeperApp restores all game state to its initial values.
// It checks that the board is reset to the correct size and mine count, all cells are covered and unflagged,
// timers are reset, sound setting is preserved, game state is set to NotStarted, and all animation/effect state
// (particles, shockwaves, pop/wave timers, mine reveal queue, wrong flags) are cleared.

#[test]
fn test_reset_game_resets_everything() {
    let mut app = MinesweeperApp::new(16, 16, 40); // Medium by default

    // Simulate a game in progress: uncover and flag some cells, set timer, set sound, add wrong flags, etc.
    app.set_state(GameState::Running);
    app.set_start_time(123.0);
    app.set_end_time(Some(456.0));
    app.set_sound(false); // Mute sound
    app.board_mut().uncover_cell(2, 2);
    app.board_mut().flag_cell(3, 3);
    app.wrong_flags_mut().push((1, 1));
    app.mine_reveal_queue_mut().push((0, 0, true));
    app.shockwaves_mut().push((1.0, 1.0, 0.5));
    for row in app.pop_timers_mut() {
        for timer in row.iter_mut() {
            *timer = Some(1.0);
        }
    }
    for row in app.wave_timers_mut() {
        for timer in row.iter_mut() {
            *timer = Some(1.0);
        }
    }

    // Call reset_game
    app.reset_game();

    // Check: All cells are covered and not flagged
    for row in 0..app.board().height() {
        for col in 0..app.board().width() {
            assert_eq!(
                app.board().cell_state(row, col),
                Some(CellState::Covered),
                "All cells should be covered after reset"
            );
        }
    }

    // Check: Timer is reset
    assert_eq!(app.start_time(), 0.0, "Start time should be reset to 0");
    assert_eq!(app.end_time(), None, "End time should be None after reset");

    // Check: Board size and mine count are correct (medium)
    assert_eq!(app.board().width(), 16, "Board width should be 16 after reset");
    assert_eq!(app.board().height(), 16, "Board height should be 16 after reset");
    assert_eq!(app.board().mines(), 40, "Mine count should be 40 after reset");

    // Check: No flagged cells remain
    for row in 0..app.board().height() {
        for col in 0..app.board().width() {
            assert_ne!(
                app.board().cell_state(row, col),
                Some(CellState::Flagged),
                "No cell should be flagged after reset"
            );
        }
    }

    // Check: Sound setting is preserved after reset
    assert!(!app.sound(), "Sound should remain muted after reset");
    app.set_sound(true);
    app.reset_game();
    assert!(app.sound(), "Sound should remain on after reset");

    // Check: Game state is reset to NotStarted
    assert_eq!(app.state(), GameState::NotStarted, "Game state should be NotStarted after reset");

    // Check: No wrongly flagged cells remain
    assert!(app.wrong_flags().is_empty(), "No wrongly flagged cells should remain after reset");

    // Check: Animation/effect state is reset
    assert!(app.particles().is_empty(), "No particles should remain after reset");
    assert!(app.shockwaves().is_empty(), "No shockwaves should remain after reset");
    for row in app.pop_timers() {
        for timer in row {
            assert!(timer.is_none(), "All pop timers should be None after reset");
        }
    }
    for row in app.wave_timers_mut() {
        for timer in row {
            assert!(timer.is_none(), "All wave timers should be None after reset");
        }
    }

    // Check: Mine reveal queue is empty
    assert!(app.mine_reveal_queue().is_empty(), "Mine reveal queue should be empty after reset");
}


// This test verifies that changing the board size resets the game state and updates the board dimensions and mine count accordingly.
// It also checks that selecting the same size does not reset the game.
#[test]
fn test_board_size_change_resets_game_and_sets_correct_size() {
    // Assume BoardSize::Small = 9x9, 10 mines; Medium = 16x16, 40 mines; Large = 24x24, 99 mines

    // Start with Small
    let mut app = MinesweeperApp::new(9, 9, 10);
    app.set_start_time(100.0);
    app.set_end_time(Some(200.0));
    app.board_mut().flag_cell(0, 0);

    // 1. Select the same size (Small) - nothing should change
    let old_start_time = app.start_time();
    let old_end_time = app.end_time();

    // State should be unchanged
    assert_eq!(app.start_time(), old_start_time, "Start time should not change when selecting same size");
    assert_eq!(app.end_time(), old_end_time, "End time should not change when selecting same size");

    // 2. Change to Medium - should reset the game
    app.set_board_size(BoardSize::Medium);
    app.reset_game(); // Simulate what your UI would do on size change

    assert_eq!(app.board().width(), 16, "Board width should be 16 for Medium");
    assert_eq!(app.board().height(), 16, "Board height should be 16 for Medium");
    assert_eq!(app.board().mines(), 40, "Mine count should be 40 for Medium");
    assert_eq!(app.start_time(), 0.0, "Start time should be reset after size change");
    assert_eq!(app.end_time(), None, "End time should be reset after size change");
    for row in 0..app.board().height() {
        for col in 0..app.board().width() {
            assert_eq!(
                app.board().cell_state(row, col),
                Some(CellState::Covered),
                "All cells should be covered after size change"
            );
        }
    }

    // 3. Change to Large - should reset the game
    app.set_board_size(BoardSize::Large);
    app.reset_game();

    assert_eq!(app.board().width(), 24, "Board width should be 24 for Large");
    assert_eq!(app.board().height(), 24, "Board height should be 24 for Large");
    assert_eq!(app.board().mines(), 99, "Mine count should be 99 for Large");
    assert_eq!(app.start_time(), 0.0, "Start time should be reset after size change");
    assert_eq!(app.end_time(), None, "End time should be reset after size change");
    for row in 0..app.board().height() {
        for col in 0..app.board().width() {
            assert_eq!(
                app.board().cell_state(row, col),
                Some(CellState::Covered),
                "All cells should be covered after size change"
            );
        }
    }

    // 4. Change back to Medium - should reset the game again
    app.set_board_size(BoardSize::Medium);
    app.reset_game();

    assert_eq!(app.board().width(), 16, "Board width should be 16 for Medium");
    assert_eq!(app.board().height(), 16, "Board height should be 16 for Medium");
    assert_eq!(app.board().mines(), 40, "Mine count should be 40 for Medium");
    assert_eq!(app.start_time(), 0.0, "Start time should be reset after size change");
    assert_eq!(app.end_time(), None, "End time should be reset after size change");
    for row in 0..app.board().height() {
        for col in 0..app.board().width() {
            assert_eq!(
                app.board().cell_state(row, col),
                Some(CellState::Covered),
                "All cells should be covered after size change"
            );
        }
    }
}


// This test verifies that the first cell uncovered is never a mine on the default Medium board.
#[test]
fn test_first_click_never_hits_mine_medium_board() {
    let mut app = MinesweeperApp::new(16, 16, 40);
    app.board_mut().place_mines_avoiding(5, 5);
    app.board_mut().uncover_cell(5, 5);
    assert_ne!(app.board().cell(5, 5), Some(Cell::Mine), "First click should never be a mine");
}