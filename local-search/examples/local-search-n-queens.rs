use std::collections::HashSet;
use std::hash::Hash;

use local_search::DecisionVariable;
use local_search::LocalSearchSolver;
use local_search::Neighborhood;
use local_search::Solution;
use local_search::Value;
use rand::prelude::SliceRandom;
use rand::SeedableRng;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct NQueensValue {
    row: i32,
}

impl Value for NQueensValue {}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct NQueensDecisionVariable {
    value: NQueensValue,

    // In the n-queens problem the column for a decision variable is fixed because we know all queens must be
    // on distinct columns.
    column: i32,
}

impl DecisionVariable for NQueensDecisionVariable {
    type V = NQueensValue;
    fn get_current_value(&self) -> &NQueensValue {
        &self.value
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct NQueenSolution {
    board_size: i32,
    variables: Vec<NQueensDecisionVariable>,
}

impl std::fmt::Debug for NQueenSolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lookup: HashSet<(usize, usize)> = self
            .variables
            .iter()
            .enumerate()
            .map(|(col, v)| (col, v.value.row as usize))
            .collect();
        let mut output = String::new();
        for row in 0..(self.board_size * 2) + 1 {
            if row % 2 == 0 {
                (0..(self.board_size * 4) + 1).for_each(|_| output += "-");
                if row != self.board_size * 2 {
                    output += "\n";
                }
                continue;
            }
            for col in 0..self.board_size {
                if lookup.contains(&(((row - 1) / 2) as usize, col as usize)) {
                    output += "| Q ";
                } else {
                    output += "|   ";
                }
                if col == self.board_size - 1 {
                    output += "|";
                }
            }
            if row != self.board_size * 2 {
                output += "\n";
            }
        }
        f.write_fmt(format_args!("{}", output))
    }
}

impl Solution for NQueenSolution {
    type D = NQueensDecisionVariable;

    fn get_hard_score(&self) -> i32 {
        self.variables.iter().map(|v| self.get_violations(v)).sum()
    }

    fn is_feasible(&self) -> bool {
        self.variables.iter().all(|v| self.get_violations(v) == 0)
    }

    fn get_violations(&self, decision_variable: &Self::D) -> i32 {
        let mut result = 0;
        for other in self
            .variables
            .iter()
            .filter(|other| decision_variable.column != other.column)
        {
            let row_diff = decision_variable.value.row - other.value.row;
            if row_diff == 0 {
                result += 1;
                continue;
            }
            let column_diff = decision_variable.column - other.column;
            if row_diff.abs() == column_diff.abs() {
                result += 1;
                continue;
            }
        }
        result
    }
}

struct NQueenNeighborhood {
    board_size: i32,
}

impl Neighborhood for NQueenNeighborhood {
    type S = NQueenSolution;
    type R = rand_pcg::Pcg64;

    fn get_initial_solution(&self) -> Self::S {
        let mut rows: Vec<i32> = (0..self.board_size).collect();
        rows.shuffle(rng);
        let variables = rows
            .iter()
            .enumerate()
            .map(|(column, row)| NQueensDecisionVariable {
                column: column as i32,
                value: NQueensValue { row: *row },
            })
            .collect();
        NQueenSolution {
            board_size: self.board_size,
            variables,
        }
    }

    fn get_local_move(&self, start: &Self::S, rng: &mut Self::R) -> Self::S {
        todo!()
    }
}

fn main() {
    println!("local search n-queens example");
    let mut rng = rand_pcg::Pcg64::seed_from_u64(42);
    let board_size = 8;
    let solver: LocalSearchSolver<NQueensValue, NQueenSolution, NQueenNeighborhood> = LocalSearchSolver::new();
    let solution = NQueenSolution::new(board_size, &mut rng);
    println!("solution:\n{:?}", solution);
    println!("solution hard score: {:?}", solution.get_hard_score());
}
