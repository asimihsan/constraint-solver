use std::{fmt::Debug, hash::Hash, marker::PhantomData};

pub trait Value: Clone + Send + PartialEq + Eq + Hash + Ord + PartialOrd + Debug {}

pub trait DecisionVariable: Clone + Send + PartialEq + Eq + Hash + Debug {
    type V: Value;
    fn get_current_value(&self) -> &Self::V;
    fn new_with_value_replacement(&self, new_value: Self::V) -> Self;
}

// -    A pure satisfaction problem moves from an infeasible configuration and tries to find any feasible
//      solution. is_feasible is false, until it is true and we're done.
// -    A pure optimization problem always has feasible solutions but move from suboptimal solutions to
//      more optimal solutions. is_feasible is always true, and we're trying to minimize get_score.
//      A constraint optimization problem combines both satisfaction and optimization.
pub trait Solution: Clone + Send + PartialEq + Eq + Hash + Debug {
    type V: Value;
    type D: DecisionVariable;

    fn get_variables(&self) -> &[Self::D];
    fn get_violations(&self, decision_variable: &Self::D) -> i32;
    fn new_solution_with_variable_replacement(
        &self,
        old_variable: &Self::D,
        new_variable: Self::D,
    ) -> Self;

    fn is_feasible(&self) -> bool {
        self.get_variables()
            .iter()
            .all(|v| self.get_violations(v) == 0)
    }

    fn get_hard_score(&self) -> i32 {
        self.get_variables()
            .iter()
            .map(|v| self.get_violations(v))
            .sum()
    }

    fn get_max_conflict_decision_variable(&self) -> &Self::D {
        self.get_variables()
            .iter()
            .max_by_key(|v| self.get_violations(v))
            .unwrap()
    }
}

pub enum LocalSearchStrategy {
    // Over all decision variables, find the decision variable with the largest number of violations. Then ensure
    // we know what all possible values of a decision variable is. Finally, change this max-conflig decision variable's
    // value such that it has a minimum number of violations. If there are multiple such values choose one at random.
    MaxMinConflict,
}

pub trait Neighborhood: Clone + Send {
    type V: Value;
    type D: DecisionVariable;
    type S: Solution;
    type R: rand::SeedableRng + ?Sized;

    fn get_initial_solution(&mut self) -> Self::S;
    fn get_local_move(&mut self, start: &Self::S) -> Self::S;
    fn get_all_possible_values(&self) -> Vec<Self::V>;
}

pub struct LocalSearchSolver<V, D, S, N>
where
    V: Value,
    D: DecisionVariable<V = V>,
    S: Solution<V = V, D = D>,
    N: Neighborhood<V = V, D = D, S = S>,
{
    phantom_v: PhantomData<V>,
    phantom_s: PhantomData<S>,
    neighborhood: N,
    best_solution: S,
    all_possible_values: Vec<V>,
}

impl<V, D, S, N> LocalSearchSolver<V, D, S, N>
where
    V: Value,
    D: DecisionVariable<V = V>,
    S: Solution<V = V, D = D>,
    N: Neighborhood<V = V, D = D, S = S>,
{
    pub fn new(mut neighborhood: N) -> Self {
        LocalSearchSolver {
            phantom_v: PhantomData,
            phantom_s: PhantomData,
            neighborhood: neighborhood.clone(),
            best_solution: neighborhood.get_initial_solution(),
            all_possible_values: neighborhood.get_all_possible_values(),
        }
    }

    pub fn _set_strategy(&mut self, _strategy: LocalSearchStrategy) {
        todo!()
    }

    pub fn _set_max_iterations(&mut self, _max_iterations: i32) {
        todo!()
    }

    pub fn iterate(&mut self) {
        println!("iterating...");
        println!("old solution hard score: {}", self.best_solution.get_hard_score());
        println!("{:?}", self.best_solution);
        let max_conflict_variable = self.best_solution.get_max_conflict_decision_variable();
        println!("max_conflict_variable: {:?}", max_conflict_variable);
        let mut new_solutions: Vec<(i32, S)> = self
            .all_possible_values
            .iter()
            .map(|v| {
                self.best_solution.new_solution_with_variable_replacement(
                    max_conflict_variable,
                    max_conflict_variable.new_with_value_replacement(v.clone()),
                )
            })
            .map(|s| (s.get_hard_score(), s))
            .collect();
        new_solutions.sort_by_key(|x| x.0);
        self.best_solution = new_solutions.first().unwrap().1.clone();
        println!("new solution hard score: {}", self.best_solution.get_hard_score());
        println!("{:?}", self.best_solution);
    }

    pub fn get_best_solution(&mut self) -> &S {
        &self.best_solution
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
