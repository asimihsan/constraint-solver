#[macro_use]
extern crate derivative;

use std::collections::HashSet;

use local_search::iterated_local_search::Perturbation;
use local_search::local_search::{
    InitialSolutionGenerator, MoveProposer, Score, Solution, SolutionScoreCalculator,
};
use rand::prelude::SliceRandom;

// In the n-queens problem the column for a decision variable is fixed because we know all queens must be
// on distinct columns.  So e.g. for a 8 x 8 board, rows[0] contains the row for the queen in the 1st
// column, rows[2] contains the row for the queen in the 2nd column, etc.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct NQueensSolution {
    rows: Vec<u64>,
}
impl Solution for NQueensSolution {}

// Print out solutions, useful for small solutions, nice-to-have.
impl std::fmt::Debug for NQueensSolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let board_size = self.rows.len();
        let lookup: HashSet<(u64, u64)> = self
            .rows
            .iter()
            .enumerate()
            .map(|(col, v)| (*v, col as u64))
            .collect();
        let mut output = String::new();
        for row in 0..(board_size * 2) + 1 {
            if row % 2 == 0 {
                (0..(board_size * 4) + 1).for_each(|_| output += "-");
                if row != board_size * 2 {
                    output += "\n";
                }
                continue;
            }
            for col in 0..board_size {
                if lookup.contains(&(((row - 1) / 2) as u64, col as u64)) {
                    output += "| Q ";
                } else {
                    output += "|   ";
                }
                if col == board_size - 1 {
                    output += "|";
                }
            }
            if row != board_size * 2 {
                output += "\n";
            }
        }
        f.write_fmt(format_args!("{}", output))
    }
}

// The number of conflicts, i.e. number of queens attacking each other. Want this to reach zero.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NQueensScore(u64);
impl Score for NQueensScore {}

#[derive(Derivative)]
#[derivative(Default)]
pub struct NQueensSolutionScoreCalculator {}

impl SolutionScoreCalculator for NQueensSolutionScoreCalculator {
    type _Solution = NQueensSolution;
    type _Score = NQueensScore;

    fn get_score(&self, solution: &Self::_Solution) -> Self::_Score {
        let mut result = 0;
        for (col1, row1) in solution.rows.iter().enumerate() {
            for (col2, row2) in solution.rows.iter().skip(col1).enumerate() {
                let row_diff = *row2 as i64 - *row1 as i64;
                if row_diff == 0 {
                    result += 1;
                    continue;
                }
                let column_diff = col2 as i64 - col1 as i64;
                if row_diff.abs() == column_diff.abs() {
                    result += 1;
                    continue;
                }
            }
        }
        NQueensScore(result)
    }
}

pub struct NQueensInitialSolutionGenerator {
    board_size: usize,
}

impl NQueensInitialSolutionGenerator {
    pub fn new(board_size: usize) -> Self {
        NQueensInitialSolutionGenerator { board_size }
    }
}

impl InitialSolutionGenerator for NQueensInitialSolutionGenerator {
    type R = rand_chacha::ChaCha20Rng;
    type Solution = NQueensSolution;

    fn generate_initial_solution(&self, rng: &mut Self::R) -> Self::Solution {
        let mut rows: Vec<u64> = (0..self.board_size).map(|x| x as u64).collect();
        rows.shuffle(rng);
        NQueensSolution { rows }
    }
}
