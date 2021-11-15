use std::hash::Hash;

use local_search::DecisionVariable;
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

impl NQueenSolution {
    pub fn new<R: rand::Rng + ?Sized>(board_size: i32, rng: &mut R) -> Self {
        let mut rows: Vec<i32> = (0..board_size).collect();
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
            board_size,
            variables,
        }
    }
}

impl std::fmt::Debug for NQueenSolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NQueenSolution").field("board_size", &self.board_size).field("variables", &self.variables).finish()
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

fn main() {
    println!("local search n-queens example");
    let mut rng = rand_pcg::Pcg64::seed_from_u64(42);
    let board_size = 8;
    let solution = NQueenSolution::new(board_size, &mut rng);
    println!("solution: {:?}", solution);
    println!("solution hard score: {:?}", solution.get_hard_score());
}
