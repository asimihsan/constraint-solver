use std::{hash::Hash, marker::PhantomData};

pub trait Value: Clone + Copy + Send + PartialEq + Eq + Hash + Ord + PartialOrd {}

pub trait DecisionVariable {
    type V: Value;
    fn get_current_value(&self) -> Self::V;
}

// -    A pure satisfacation problem moves from an infeasible configuration and tries to find any feasible
//      solution. is_feasible is false, until it is true and we're done.
// -    A pure optimization problem always has feasible solutions but move from suboptimal solutions to
//      more optimal solutions. is_feasible is always true, and we're trying to minimize get_score.
//      A constraint optimization problem combines both satisfaction and optimization.
pub trait Solution: Clone + Send + PartialEq + Eq + Hash {
    type D: DecisionVariable;
    fn is_feasible(&self) -> bool;
    fn get_violations(&self, decision_variable: &Self::D) -> i64;
}

pub enum LocalSearchStrategy {
    // Over all decision variables, find the decision variable with the largest number of violations. Then ensure
    // we know what all possible values of a decision variable is. Finally, change this max-conflig decision variable's
    // value such that it has a minimum number of violations. If there are multiple such values choose one at random.
    MaxMinConflict,
}

pub trait Neighborhood {
    type S: Solution;
    fn get_initial_solution(&self) -> Self::S;
    fn get_local_move(&self, start: &Self::S) -> Self::S;
}

pub struct LocalSearchSolver<V, S, N>
where
    V: Value,
    S: Solution,
    N: Neighborhood,
{
    phantom_v: PhantomData<V>,
    phantom_s: PhantomData<S>,
    phantom_n: PhantomData<N>,
}

impl<V, S, N> LocalSearchSolver<V, S, N>
where
    V: Value,
    N: Neighborhood,
    S: Solution,
{
    fn _set_strategy(&mut self, _strategy: LocalSearchStrategy) {
        todo!()
    }

    fn _set_max_iterations(&mut self, _max_iterations: i64) {
        todo!()
    }

    fn _iterate(&mut self) {
        todo!()
    }

    fn _get_initial_solution(&mut self) -> S {
        todo!()
    }

    fn _get_best_solution(&mut self) -> S {
        todo!()
    }

    fn _get_all_possible_values(&self) -> Vec<V> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
