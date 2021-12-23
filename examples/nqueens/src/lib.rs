#[macro_use]
extern crate derivative;

use std::collections::HashMap;
use std::collections::HashSet;

use local_search::iterated_local_search::Perturbation;
use local_search::local_search::{
    InitialSolutionGenerator, MoveProposer, Score, Solution, SolutionScoreCalculator,
};
use rand::prelude::SliceRandom;
use rand::Rng;

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
impl Score for NQueensScore {
    /// If there are no conflicts, i.e. a score of zero, this is the best score.
    fn is_best(&self) -> bool {
        self.0 == 0
    }
}

fn get_col_scores(solution: &NQueensSolution) -> HashMap<usize, u64> {
    let mut result = HashMap::with_capacity(solution.rows.len());
    for col in 0..solution.rows.len() {
        result.insert(col, 0);
    }
    for (col1, row1) in solution.rows.iter().enumerate() {
        for (col2, row2) in solution.rows.iter().enumerate().skip(col1 + 1) {
            let row_diff = *row2 as i64 - *row1 as i64;
            let column_diff = col2 as i64 - col1 as i64;
            if row_diff == 0 || row_diff.abs() == column_diff.abs() {
                *result.entry(col1).or_insert(0) += 1;
                *result.entry(col2).or_insert(0) += 1;
            }
        }
    }
    result
}

#[cfg(test)]
mod get_col_scores_tests {
    use super::*;

    #[test]
    fn test_all_same_row() {
        let solution = NQueensSolution {
            rows: vec![0, 0, 0, 0],
        };
        let scores = get_col_scores(&solution);
        println!("solution:\n{:?}\n, scores: {:?}", solution, scores);
        assert_eq!(solution.rows.len(), scores.len());
        assert_eq!(3, *scores.get(&0).unwrap());
        assert_eq!(3, *scores.get(&1).unwrap());
        assert_eq!(3, *scores.get(&2).unwrap());
        assert_eq!(3, *scores.get(&3).unwrap());
    }

    #[test]
    fn test_best_solution_has_zero_score() {
        let solution = NQueensSolution {
            rows: vec![1, 3, 0, 2],
        };
        let scores = get_col_scores(&solution);
        println!("solution:\n{:?}\n, scores: {:?}", solution, scores);
        assert_eq!(solution.rows.len(), scores.len());
        assert_eq!(0, *scores.get(&0).unwrap());
        assert_eq!(0, *scores.get(&1).unwrap());
        assert_eq!(0, *scores.get(&2).unwrap());
        assert_eq!(0, *scores.get(&3).unwrap());
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct NQueensSolutionScoreCalculator {}

impl SolutionScoreCalculator for NQueensSolutionScoreCalculator {
    type _Solution = NQueensSolution;
    type _Score = NQueensScore;

    fn get_score(&self, solution: &Self::_Solution) -> Self::_Score {
        let row_scores = get_col_scores(solution);
        NQueensScore(row_scores.values().sum())
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
        let mut rows: Vec<u64> = (0..usize::from(self.board_size)).map(|x| x as u64).collect();
        rows.shuffle(rng);
        NQueensSolution { rows }
    }
}

pub struct NQueensMoveProposer {
    board_size: usize,
}

impl NQueensMoveProposer {
    pub fn new(board_size: usize) -> Self {
        Self { board_size }
    }
}

impl MoveProposer for NQueensMoveProposer {
    type R = rand_chacha::ChaCha20Rng;
    type Solution = NQueensSolution;

    fn iter_local_moves(
        &self,
        start: &Self::Solution,
        rng: &mut Self::R,
    ) -> Box<dyn Iterator<Item = Self::Solution>> {
        let mut cols_with_conflicts: Vec<(usize, u64)> = get_col_scores(start)
            .into_iter()
            .filter(|(_row, score)| *score != 0)
            .collect();
        cols_with_conflicts.shuffle(rng);
        // println!("start: {:?}", start);
        // println!("cols_with_conflicts: {:?}", cols_with_conflicts);
        let random_cols = if cols_with_conflicts.is_empty() {
            None
        } else {
            // let amount = (start.rows.len() / 20).clamp(1, cols_with_conflicts.len());
            // let cols: Vec<usize> = cols_with_conflicts
            //     .choose_multiple_weighted(rng, amount, |(_col, score)| *score as f64)
            //     .unwrap()
            //     .map(|(col, _score)| *col)
            //     .collect();
            // let num_cols = rng.gen_range(1..=cols.len());
            // let cols = cols.choose_multiple(rng, num_cols).map(|col| *col).collect();
            Some(cols_with_conflicts.iter().map(|(col, _score)| *col).collect())
        };
        struct MoveIterator {
            board_size: u64,
            cols: Option<Vec<usize>>,
            current_col: usize,
            current_value: u64,
            solution: NQueensSolution,
        }

        impl Iterator for MoveIterator {
            type Item = NQueensSolution;

            fn next(&mut self) -> Option<Self::Item> {
                if self.current_value >= self.board_size {
                    self.current_col += 1;
                    self.current_value = 0;
                }
                if let Some(cols) = &self.cols {
                    if self.current_col >= cols.len() {
                        return None;
                    }
                    let col = cols[self.current_col];
                    let mut new_solution = self.solution.clone();
                    new_solution.rows[col] = self.current_value;
                    self.current_value += 1;
                    Some(new_solution)
                } else {
                    None
                }
            }
        }

        Box::new(MoveIterator {
            board_size: start.rows.len() as u64,
            cols: random_cols,
            current_col: 0,
            current_value: 0,
            solution: start.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NQueensPerturbationStrategy {
    ChangeSubset,
    DoNothing,
}

pub struct NQueensPerturbation {
    strategy: Vec<(NQueensPerturbationStrategy, u64)>,
}

impl NQueensPerturbation {
    pub fn new(strategy: Vec<(NQueensPerturbationStrategy, u64)>) -> Self {
        Self { strategy }
    }
}

impl Default for NQueensPerturbation {
    fn default() -> Self {
        Self {
            strategy: vec![
                (NQueensPerturbationStrategy::ChangeSubset, 100),
                (NQueensPerturbationStrategy::DoNothing, 10),
            ],
        }
    }
}

impl Perturbation for NQueensPerturbation {
    type _R = rand_chacha::ChaCha20Rng;
    type _Solution = NQueensSolution;
    type _Score = NQueensScore;
    type _SSC = NQueensSolutionScoreCalculator;

    fn propose_new_starting_solution(
        &mut self,
        current: &local_search::local_search::ScoredSolution<Self::_Solution, Self::_Score>,
        history: &local_search::local_search::History<Self::_R, Self::_Solution, Self::_Score>,
        rng: &mut Self::_R,
    ) -> Self::_Solution {
        let current_strategy = self.strategy.choose_weighted(rng, |s| s.1).unwrap().0.clone();
        let mut new_solution = current.solution.clone();
        match current_strategy {
            NQueensPerturbationStrategy::ChangeSubset => {
                let board_size = current.solution.rows.len() as u64;
                let mut rows: Vec<u64> = (0..board_size).collect();
                rows.shuffle(rng);
                let number_of_rows_to_alter = match history.is_best_solution(current) {
                    true => rng.gen_range(1..=(board_size / 20).clamp(1, board_size)),
                    false => rng.gen_range(1..=(board_size / 2).clamp(1, board_size)),
                };
                let rows_to_alter: Vec<u64> =
                    rows.into_iter().take(number_of_rows_to_alter as usize).collect();
                for i in rows_to_alter {
                    let new_col = rng.gen_range(0..board_size);
                    new_solution.rows[i as usize] = new_col;
                }
                // println!("change subset perturbed {:?} to {:?}", &current.solution, &new_solution);
                new_solution
            }
            NQueensPerturbationStrategy::DoNothing => new_solution,
        }
    }
}
