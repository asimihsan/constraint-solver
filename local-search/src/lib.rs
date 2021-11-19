use std::{collections::BTreeSet, fmt::Debug, hash::Hash, marker::PhantomData};

use linked_hash_set::LinkedHashSet;
use rand::prelude::SliceRandom;

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
pub trait Solution: Clone + Send + PartialEq + Eq + Hash + Debug + Ord + PartialOrd {
    type D: DecisionVariable;

    fn get_variables(&self) -> &[Self::D];
    fn get_violations(&self, decision_variable: &Self::D) -> u64;

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

    fn get_hard_score(&self) -> u64 {
        self.get_variables()
            .iter()
            .map(|v| self.get_violations(v))
            .sum()
    }

    fn get_max_conflict_decision_variables(&self) -> Vec<&Self::D> {
        let all_violations_values: Vec<(u64, &Self::D)> = self
            .get_variables()
            .iter()
            .map(|v| (self.get_violations(v), v))
            .collect();
        let max_violations_value = all_violations_values
            .iter()
            .map(|(value, _v)| value)
            .max()
            .unwrap();
        let max_conflict_variables: Vec<&Self::D> = all_violations_values
            .iter()
            .filter(|(value, _v)| value == max_violations_value)
            .map(|(_value, v)| *v)
            .collect();
        max_conflict_variables
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LocalSearchStrategy {
    // Over all decision variables, find the decision variable with the largest number of violations. Then
    // ensure we know what all possible values of a decision variable is. Finally, change this max-conflig
    // decision variable's value such that it has a minimum number of violations. If there are multiple such
    // values choose one at random.
    MaxMinConflict,

    // Over all decision variables, pick a random decision variable. Then ensure we know what all possible
    // values of a decision variable is. Finally, change this decision variable's value such that it has a
    // minimum number of violations. If there are multiple such values choose one at random.
    MinConflict,

    // Choose random decision variable and assign random value.
    Random,

    // Completely restart
    Restart,

    // Choose randomly from the best set
    ChooseFromBestSet,
}

pub trait Neighborhood: Clone + Send {
    type V: Value;
    type D: DecisionVariable;
    type S: Solution;
    type R: rand::Rng;

    fn get_initial_solution(&mut self) -> Self::S;
    fn get_local_move(&mut self, start: &Self::S) -> Self::S;
    fn get_all_possible_values(&self) -> Vec<Self::V>;
}

struct BestSolutions<S, R>
where
    S: Solution,
    R: rand::Rng,
{
    best_solutions: BTreeSet<S>,
    capacity: usize,
    phantom_r: PhantomData<R>,
}

impl<S, R> BestSolutions<S, R>
where
    S: Solution,
    R: rand::Rng,
{
    pub fn new(capacity: usize) -> Self {
        BestSolutions {
            best_solutions: Default::default(),
            capacity,
            phantom_r: PhantomData,
        }
    }

    pub fn insert(&mut self, candidate_solution: &S) {
        if self.best_solutions.len() < self.capacity {
            self.best_solutions.insert(candidate_solution.clone());
            return;
        }

        // TODO better heuristic for creating a diverse best solution set even if the candidate solution has a worse
        // score.
        let worst_solution = self.best_solutions.iter().next_back().unwrap().clone();
        if candidate_solution.get_hard_score() <= worst_solution.get_hard_score() {
            self.best_solutions.remove(&worst_solution);
            self.best_solutions.insert(candidate_solution.clone());
        }
    }

    pub fn get_random(&mut self, rng: &mut R) -> Option<S> {
        if self.best_solutions.is_empty() {
            return None;
        }
        let best_solutions_vec: Vec<S> = self.best_solutions.iter().cloned().collect();
        let random_best_solution = best_solutions_vec.choose(rng).unwrap().clone();
        Some(random_best_solution)
    }

    pub fn _clear(&mut self) {
        self.best_solutions.clear();
    }
}

impl<S, R> Default for BestSolutions<S, R>
where
    S: Solution,
    R: rand::Rng,
{
    fn default() -> Self {
        BestSolutions::new(16)
    }
}

pub struct LocalSearchSolver<V, D, S, N, R>
where
    V: Value,
    D: DecisionVariable<V = V>,
    S: Solution<D = D>,
    N: Neighborhood<V = V, D = D, S = S>,
    R: rand::Rng,
{
    phantom_v: PhantomData<V>,
    phantom_s: PhantomData<S>,
    neighborhood: N,
    best_solution: S,
    all_possible_values: Vec<V>,
    rng: R,
    strategy: Vec<(LocalSearchStrategy, u64)>,
    same_score_iteration_count: usize,
    iteration_count: u64,
    tabu_set: LinkedHashSet<S>,
    tabu_capacity: u64,
    best_solutions: BestSolutions<S, R>,
}

impl<V, D, S, N, R> LocalSearchSolver<V, D, S, N, R>
where
    V: Value,
    D: DecisionVariable<V = V>,
    S: Solution<D = D>,
    N: Neighborhood<V = V, D = D, S = S>,
    R: rand::Rng,
{
    pub fn new(mut neighborhood: N, rng: R) -> Self {
        let initial_solution = neighborhood.get_initial_solution();
        let all_possible_values = neighborhood.get_all_possible_values();
        let solution_space_size =
            initial_solution.get_variables().len() as u64 * all_possible_values.len() as u64;
        LocalSearchSolver {
            phantom_v: PhantomData,
            phantom_s: PhantomData,
            neighborhood: neighborhood.clone(),
            best_solution: initial_solution,
            all_possible_values: neighborhood.get_all_possible_values(),
            rng,
            strategy: vec![
                (LocalSearchStrategy::ChooseFromBestSet, 5),
                (LocalSearchStrategy::Restart, 0),
                (LocalSearchStrategy::MinConflict, 100),
                (LocalSearchStrategy::MaxMinConflict, 680),
                (LocalSearchStrategy::Random, 200),
            ],
            same_score_iteration_count: 0,
            iteration_count: 0,
            tabu_set: Default::default(),
            tabu_capacity: solution_space_size,
            best_solutions: Default::default(),
        }
    }

    pub fn _set_strategy(&mut self, _strategy: LocalSearchStrategy) {
        todo!()
    }

    pub fn _set_max_iterations(&mut self, _max_iterations: i32) {
        todo!()
    }

    pub fn iterate(&mut self) {
        println!("iteration count {}...", self.iteration_count);
        self.iteration_count += 1;
        let old_score = self.best_solution.get_hard_score();
        println!("old solution hard score: {}", old_score);
        // println!("{:?}", self.best_solution);

        let current_strategy = self
            .strategy
            .choose_weighted(&mut self.rng, |s| s.1)
            .unwrap()
            .0
            .clone();
        println!("current strategy: {:?}", current_strategy);
        let new_solution = match current_strategy {
            LocalSearchStrategy::MaxMinConflict | LocalSearchStrategy::MinConflict => {
                let variable_to_use = match current_strategy {
                    LocalSearchStrategy::MaxMinConflict => {
                        let max_conflict_variables =
                            self.best_solution.get_max_conflict_decision_variables();
                        // println!("max_conflict_variables: {:?}", max_conflict_variables);
                        let max_conflict_variable =
                            max_conflict_variables.choose(&mut self.rng).unwrap();
                        // println!("max_conflict_variable: {:?}", max_conflict_variable);
                        max_conflict_variable.clone()
                    }
                    LocalSearchStrategy::MinConflict => {
                        let random_variable = self
                            .best_solution
                            .get_variables()
                            .choose(&mut self.rng)
                            .unwrap();
                        // println!("random_variable: {:?}", random_variable);
                        random_variable
                    }
                    _ => todo!(),
                };
                let mut new_solutions: Vec<(u64, S)> = self
                    .all_possible_values
                    .iter()
                    .map(|v| {
                        self.best_solution.new_solution_with_variable_replacement(
                            variable_to_use,
                            variable_to_use.new_with_value_replacement(v.clone()),
                        )
                    })
                    .filter(|s| !self.tabu_set.contains(s))
                    .map(|s| (s.get_hard_score(), s))
                    .collect();
                new_solutions.sort_by_key(|x| x.0);
                if new_solutions.is_empty() {
                    return;
                }
                let new_solution = new_solutions.first().unwrap().1.clone();
                new_solution
            }
            LocalSearchStrategy::Random => {
                let random_count = 1;
                let mut new_solution = self.best_solution.clone();
                for _ in 0..random_count {
                    let random_variable =
                        new_solution.get_variables().choose(&mut self.rng).unwrap();
                    let random_value = self.all_possible_values.choose(&mut self.rng).unwrap();
                    new_solution = new_solution.new_solution_with_variable_replacement(
                        random_variable,
                        random_variable.new_with_value_replacement(random_value.clone()),
                    );
                }
                new_solution
            }
            LocalSearchStrategy::Restart => self.neighborhood.get_initial_solution(),
            LocalSearchStrategy::ChooseFromBestSet => {
                let random_from_best_set = self.best_solutions.get_random(&mut self.rng);
                if random_from_best_set.is_none() {
                    return;
                }
                random_from_best_set.unwrap().clone()
            }
        };

        if current_strategy != LocalSearchStrategy::ChooseFromBestSet && self.tabu_set.contains(&new_solution) {
            println!("skip tabu solution");
            return;
        };

        self.best_solution = new_solution.clone();
        let new_score = self.best_solution.get_hard_score();
        println!("new solution hard score: {}", new_score);

        if old_score == new_score {
            self.same_score_iteration_count += 1;
        } else {
            self.same_score_iteration_count = 0;
        }

        if self.iteration_count % self.tabu_capacity * 10 == 0 {
            println!("*** tabu clear ***");
            self.tabu_set.clear();
        } else if self.tabu_set.len() as u64 >= self.tabu_capacity {
            self.tabu_set.pop_front();
        }
        self.tabu_set.insert(new_solution);

        self.best_solutions.insert(&self.best_solution);

        // println!("{:?}", self.best_solution);
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
