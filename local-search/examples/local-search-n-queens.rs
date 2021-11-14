use std::hash::Hash;

use local_search::DecisionVariable;
use local_search::Solution;
use local_search::Value;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NQueensValue {
    row: i64,
}

impl Value for NQueensValue {}

struct NQueensDecisionVariable {
    value: NQueensValue,
}

impl DecisionVariable for NQueensDecisionVariable {
    type V = NQueensValue;
    fn get_current_value(&self) -> NQueensValue {
        self.value
    }
}

struct NQueenSolution {
    board_size: i64,
    variables: Vec<NQueensDecisionVariable>,
}

impl Solution for NQueenSolution {
    type D = NQueensDecisionVariable;

    fn is_feasible(&self) -> bool {
        todo!()
    }

    fn get_violations(&self, decision_variable: &Self::D) -> i64 {
        todo!()
    }
}

fn main() {
    println!("local search n-queens example");
}
