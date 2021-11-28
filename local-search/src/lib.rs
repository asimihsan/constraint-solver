#[macro_use]
extern crate derivative;

mod ackley;
pub mod local_search;
pub mod iterated_local_search;

// use std::{fmt::Debug, marker::PhantomData};

// use hashlink::LinkedHashSet;
// use crate::local_search::Solution;

// pub trait SearchHistory {
//     type S: Solution;

//     fn add_to_tabu(&mut self, solution: &Self::S);
//     fn is_tabu(&self, solution: &Self::S) -> bool;
// }

// pub enum AcceptanceCriterion {
//     // The new local minima solution must be better (strictly lower) in cost than the current solution.
//     Better,

//     // The new local minima solution is randomly accepted or rejected.
//     RandomWalk,
// }

// pub trait MoveAcceptor {
//     type R: rand::Rng;
//     type S: Solution;
//     type SH: SearchHistory;

//     fn accept_move(
//         &self,
//         start: &Self::S,
//         end: &Self::S,
//         history: &Self::SH,
//         acceptance_criterion: &AcceptanceCriterion,
//         rng: &mut Self::R,
//     ) -> bool;
// }

// pub trait LocalSearch {
//     type R: rand::Rng;
//     type S: Solution;

//     fn local_search(&self, start: &Self::S) -> Self::S;
// }



// #[derive(Clone, Debug, Eq, PartialEq)]
// pub enum LocalSearchStrategy {
//     // Over all decision variables, find the decision variable with the largest number of violations. Then
//     // ensure we know what all possible values of a decision variable is. Finally, change this max-conflig
//     // decision variable's value such that it has a minimum number of violations. If there are multiple such
//     // values choose one at random.
//     MaxMinConflict,

//     // Over all decision variables, pick a random decision variable. Then ensure we know what all possible
//     // values of a decision variable is. Finally, change this decision variable's value such that it has a
//     // minimum number of violations. If there are multiple such values choose one at random.
//     MinConflict,

//     // Choose random decision variable and assign random value.
//     Random,

//     // Completely restart
//     Restart,

//     // Choose randomly from the best set
//     ChooseFromBestSet,
// }

// pub trait Explorer {
//     type S: Solution;
//     type R: rand::Rng;

//     fn get_initial_solution(&mut self) -> Self::S;

//     // fn _get_local_move(
//     //     &mut self,
//     //     start: &Self::S,
//     //     strategy: &LocalSearchStrategy,
//     //     tabu_set: &LinkedHashSet<Self::S>,
//     //     best_solutions: &BestSolutions<Self::S, Self::R>,
//     //     rng: &mut Self::R,
//     // ) -> Self::S {
//     //     match strategy {
//     //         LocalSearchStrategy::MaxMinConflict | LocalSearchStrategy::MinConflict => {
//     //             let variable_to_use = match strategy {
//     //                 LocalSearchStrategy::MaxMinConflict => {
//     //                     let max_conflict_variables = start.get_max_conflict_decision_variables();
//     //                     // println!("max_conflict_variables: {:?}", max_conflict_variables);
//     //                     let max_conflict_variable = max_conflict_variables.choose(rng).unwrap();
//     //                     // println!("max_conflict_variable: {:?}", max_conflict_variable);
//     //                     max_conflict_variable.clone()
//     //                 }
//     //                 LocalSearchStrategy::MinConflict => {
//     //                     let random_variable = start.get_variables().choose(rng).unwrap();
//     //                     // println!("random_variable: {:?}", random_variable);
//     //                     random_variable
//     //                 }
//     //                 _ => todo!(),
//     //             };
//     //             let mut new_solutions: Vec<(u64, Self::S)> = self
//     //                 .all_possible_values
//     //                 .iter()
//     //                 .map(|v| {
//     //                     start.new_solution_with_variable_replacement(
//     //                         variable_to_use,
//     //                         variable_to_use.new_with_value_replacement(v.clone()),
//     //                     )
//     //                 })
//     //                 .filter(|s| !tabu_set.contains(s))
//     //                 .map(|s| (s.get_hard_score(), s))
//     //                 .collect();
//     //             new_solutions.sort_by_key(|x| x.0);
//     //             if new_solutions.is_empty() {
//     //                 return;
//     //             }
//     //             let new_solution = new_solutions.first().unwrap().1.clone();
//     //             new_solution
//     //         }
//     //         LocalSearchStrategy::Random => {
//     //             let random_count = 1;
//     //             let mut new_solution = start.clone();
//     //             for _ in 0..random_count {
//     //                 let random_variable = new_solution.get_variables().choose(rng).unwrap();
//     //                 let random_value = self.all_possible_values.choose(rng).unwrap();
//     //                 new_solution = new_solution.new_solution_with_variable_replacement(
//     //                     random_variable,
//     //                     random_variable.new_with_value_replacement(random_value.clone()),
//     //                 );
//     //             }
//     //             new_solution
//     //         }
//     //         LocalSearchStrategy::Restart => self.get_initial_solution(),
//     //         LocalSearchStrategy::ChooseFromBestSet => {
//     //             let random_from_best_set = best_solutions.get_random(rng);
//     //             if random_from_best_set.is_none() {
//     //                 return;
//     //             }
//     //             random_from_best_set.unwrap().clone()
//     //         }
//     //     }
//     // }
// }

// // struct BestSolutions<S, R>
// // where
// //     S: Solution,
// //     R: rand::Rng,
// // {
// //     best_solutions: BTreeSet<S>,
// //     capacity: usize,
// //     phantom_r: PhantomData<R>,
// // }

// // impl<S, R> BestSolutions<S, R>
// // where
// //     S: Solution,
// //     R: rand::Rng,
// // {
// //     pub fn new(capacity: usize) -> Self {
// //         BestSolutions {
// //             best_solutions: Default::default(),
// //             capacity,
// //             phantom_r: PhantomData,
// //         }
// //     }

// //     pub fn insert(&mut self, candidate_solution: &S) {
// //         if self.best_solutions.len() < self.capacity {
// //             self.best_solutions.insert(candidate_solution.clone());
// //             return;
// //         }

// //         // TODO better heuristic for creating a diverse best solution set even if the candidate solution has a worse
// //         // score.
// //         let worst_solution = self.best_solutions.iter().next_back().unwrap().clone();
// //         if candidate_solution.get_hard_score() <= worst_solution.get_hard_score() {
// //             self.best_solutions.remove(&worst_solution);
// //             self.best_solutions.insert(candidate_solution.clone());
// //         }
// //     }

// //     pub fn get_random(&mut self, rng: &mut R) -> Option<S> {
// //         if self.best_solutions.is_empty() {
// //             return None;
// //         }
// //         let best_solutions_vec: Vec<S> = self.best_solutions.iter().cloned().collect();
// //         let random_best_solution = best_solutions_vec.choose(rng).unwrap().clone();
// //         Some(random_best_solution)
// //     }

// //     pub fn get_best(&self) -> Option<S> {
// //         if self.best_solutions.is_empty() {
// //             return None;
// //         }
// //         Some(self.best_solutions.iter().next().unwrap().clone())
// //     }

// //     pub fn _clear(&mut self) {
// //         self.best_solutions.clear();
// //     }
// // }

// // impl<S, R> Default for BestSolutions<S, R>
// // where
// //     S: Solution,
// //     R: rand::Rng,
// // {
// //     fn default() -> Self {
// //         BestSolutions::new(16)
// //     }
// // }

// pub struct LocalSearchSolver<V, S, R, E>
// where
//     S: Solution,
//     R: rand::Rng,
//     E: Explorer<S = S, R = R>,
// {
//     phantom_v: PhantomData<V>,
//     phantom_s: PhantomData<S>,
//     explorer: E,
//     current_solution: S,
//     rng: R,
//     strategy: Vec<(LocalSearchStrategy, u64)>,
//     same_score_iteration_count: usize,
//     iteration_count: u64,
//     tabu_set: LinkedHashSet<S>,
//     tabu_capacity: u64,
// }

// impl<V, S, R, E> LocalSearchSolver<V, S, R, E>
// where
//     S: Solution,
//     R: rand::Rng,
//     E: Explorer<S = S, R = R>,
// {
//     pub fn new(mut explorer: E, rng: R) -> Self {
//         let initial_solution = explorer.get_initial_solution();
//         LocalSearchSolver {
//             phantom_v: PhantomData,
//             phantom_s: PhantomData,
//             explorer,
//             current_solution: initial_solution,
//             rng,
//             strategy: vec![
//                 (LocalSearchStrategy::ChooseFromBestSet, 5),
//                 (LocalSearchStrategy::Restart, 0),
//                 (LocalSearchStrategy::MinConflict, 100),
//                 (LocalSearchStrategy::MaxMinConflict, 680),
//                 (LocalSearchStrategy::Random, 200),
//             ],
//             same_score_iteration_count: 0,
//             iteration_count: 0,
//             tabu_set: Default::default(),
//             tabu_capacity: 10_000,
//         }
//     }

//     pub fn _set_strategy(&mut self, _strategy: LocalSearchStrategy) {
//         todo!()
//     }

//     pub fn _set_max_iterations(&mut self, _max_iterations: i32) {
//         todo!()
//     }

//     pub fn iterate(&mut self) {
//         // println!("iteration count {}...", self.iteration_count);
//         // self.iteration_count += 1;
//         // let old_score = self.current_solution.get_hard_score();
//         // println!("old solution hard score: {}", old_score);
//         // // println!("{:?}", self.best_solution);

//         // let current_strategy = self
//         //     .strategy
//         //     .choose_weighted(&mut self.rng, |s| s.1)
//         //     .unwrap()
//         //     .0
//         //     .clone();
//         // println!("current strategy: {:?}", current_strategy);
//         // let new_solution = self.explorer._get_local_move(
//         //     &self.current_solution,
//         //     &current_strategy,
//         //     &self.tabu_set,
//         //     &self.best_solutions,
//         //     &mut self.rng,
//         // );

//         // if current_strategy != LocalSearchStrategy::ChooseFromBestSet
//         //     && self.tabu_set.contains(&new_solution)
//         // {
//         //     println!("skip tabu solution");
//         //     return;
//         // };

//         // self.current_solution = new_solution.clone();
//         // let new_score = self.current_solution.get_hard_score();
//         // println!("new solution hard score: {}", new_score);

//         // if old_score == new_score {
//         //     self.same_score_iteration_count += 1;
//         // } else {
//         //     self.same_score_iteration_count = 0;
//         // }

//         // if self.iteration_count % self.tabu_capacity * 10 == 0 {
//         //     println!("*** tabu clear ***");
//         //     self.tabu_set.clear();
//         // } else if self.tabu_set.len() as u64 >= self.tabu_capacity {
//         //     self.tabu_set.pop_front();
//         // }
//         // self.tabu_set.insert(new_solution);

//         // self.best_solutions.insert(&self.current_solution);

//         // // println!("{:?}", self.best_solution);
//     }

//     // pub fn get_best_solution(&self) -> S {
//     //     self.best_solutions.get_best().unwrap()
//     // }
// }

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         let result = 2 + 2;
//         assert_eq!(result, 4);
//     }
// }
