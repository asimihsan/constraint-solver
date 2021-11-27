// #[macro_use]
// extern crate derivative;

// use std::collections::HashSet;
// use std::hash::Hash;

// use local_search::LocalSearchSolver;
// use local_search::Solution;
// use local_search::Variable;
// use rand::prelude::SliceRandom;
// use rand::SeedableRng;

// use blake2::digest::{Update, VariableOutput};
// use blake2::VarBlake2b;

// #[derive(Clone, PartialEq, Eq, Hash, Debug)]
// struct NQueensVariable {
//     row: u64,

//     // In the n-queens problem the column for a decision variable is fixed because we know all queens must be
//     // on distinct columns.
//     column: u64,
// }

// impl Variable for NQueensDecisionVariable {
//     type V = NQueensValue;
//     fn get_current_value(&self) -> &NQueensValue {
//         &self.value
//     }
//     fn new_with_value_replacement(&self, new_value: Self::V) -> Self {
//         NQueensDecisionVariable {
//             value: new_value,
//             column: self.column,
//         }
//     }
// }

// #[derive(Derivative)]
// #[derivative(Clone, PartialEq, Eq, Hash)]
// struct NQueenSolution {
//     board_size: u64,
//     variables: Vec<NQueensDecisionVariable>,
// }

// impl PartialOrd for NQueenSolution {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         self.get_hard_score().partial_cmp(&other.get_hard_score())
//     }
// }

// impl Ord for NQueenSolution {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.get_hard_score().cmp(&other.get_hard_score())
//     }
// }

// impl NQueenSolution {
//     pub fn new(board_size: u64, variables: Vec<NQueensDecisionVariable>) -> Self {
//         NQueenSolution {
//             board_size,
//             variables,
//         }
//     }
// }

// impl std::fmt::Debug for NQueenSolution {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let lookup: HashSet<(usize, usize)> = self
//             .variables
//             .iter()
//             .enumerate()
//             .map(|(col, v)| (v.value.row as usize, col))
//             .collect();
//         let mut output = String::new();
//         for row in 0..(self.board_size * 2) + 1 {
//             if row % 2 == 0 {
//                 (0..(self.board_size * 4) + 1).for_each(|_| output += "-");
//                 if row != self.board_size * 2 {
//                     output += "\n";
//                 }
//                 continue;
//             }
//             for col in 0..self.board_size {
//                 if lookup.contains(&(((row - 1) / 2) as usize, col as usize)) {
//                     output += "| Q ";
//                 } else {
//                     output += "|   ";
//                 }
//                 if col == self.board_size - 1 {
//                     output += "|";
//                 }
//             }
//             if row != self.board_size * 2 {
//                 output += "\n";
//             }
//         }
//         f.write_fmt(format_args!("{}", output))
//     }
// }

// impl Solution for NQueenSolution {
//     type D = NQueensDecisionVariable;

//     fn get_variables(&self) -> &[Self::D] {
//         self.variables.as_ref()
//     }

//     fn get_violations(&self, decision_variable: &Self::D) -> u64 {
//         let mut result = 0;
//         for other in self
//             .get_variables()
//             .iter()
//             .filter(|other| decision_variable.column != other.column)
//         {
//             let row_diff = decision_variable.value.row as i64 - other.value.row as i64;
//             if row_diff == 0 {
//                 result += 1;
//                 continue;
//             }
//             let column_diff = decision_variable.column as i64 - other.column as i64;
//             if row_diff.abs() == column_diff.abs() {
//                 result += 1;
//                 continue;
//             }
//         }
//         result
//     }

//     fn new_solution_with_variable_replacement(
//         &self,
//         old_variable: &Self::D,
//         new_variable: Self::D,
//     ) -> Self {
//         NQueenSolution::new(
//             self.board_size,
//             self.get_variables()
//                 .iter()
//                 .map(|old_v| {
//                     if old_v == old_variable {
//                         new_variable.clone()
//                     } else {
//                         old_v.clone()
//                     }
//                 })
//                 .collect(),
//         )
//     }
// }

// #[derive(Clone)]
// struct NQueenNeighborhood {
//     board_size: u64,
//     rng: <NQueenNeighborhood as Neighborhood>::R,
// }

// impl NQueenNeighborhood {
//     pub fn new(board_size: u64, rng: <NQueenNeighborhood as Neighborhood>::R) -> Self {
//         NQueenNeighborhood { board_size, rng }
//     }
// }

// impl Neighborhood for NQueenNeighborhood {
//     type V = NQueensValue;
//     type D = NQueensDecisionVariable;
//     type S = NQueenSolution;
//     type R = rand_chacha::ChaCha20Rng;

//     fn get_initial_solution(&mut self) -> Self::S {
//         let mut rows: Vec<u64> = (0..self.board_size).collect();
//         rows.shuffle(&mut self.rng);
//         let variables = rows
//             .iter()
//             .enumerate()
//             .map(|(column, row)| NQueensDecisionVariable {
//                 column: column as u64,
//                 value: NQueensValue { row: *row },
//             })
//             .collect();
//         NQueenSolution::new(self.board_size, variables)
//     }

//     fn get_local_move(&mut self, _start: &Self::S) -> Self::S {
//         todo!()
//     }

//     fn get_all_possible_values(
//         &self,
//     ) -> Vec<<<<NQueenNeighborhood as Neighborhood>::S as Solution>::D as DecisionVariable>::V>
//     {
//         (0..self.board_size)
//             .map(|i| NQueensValue { row: i })
//             .collect::<Vec<NQueensValue>>()
//     }
// }

// fn main() {
//     println!("local search n-queens example");
//     let matches = clap::App::new("Local Search N-Queens Example")
//         .version("1.0")
//         .arg(
//             clap::Arg::with_name("seed")
//                 .short("s")
//                 .long("seed")
//                 .value_name("STRING")
//                 .help("Random seeed, any string")
//                 .required(false)
//                 .default_value("42")
//                 .takes_value(true),
//         )
//         .arg(
//             clap::Arg::with_name("board_size")
//                 .short("b")
//                 .long("board-size")
//                 .value_name("INT")
//                 .help("Board size")
//                 .required(false)
//                 .default_value("8")
//                 .takes_value(true)
//                 .validator(|input| {
//                     if let Err(err) = input.parse::<u64>() {
//                         return Err(err.to_string());
//                     }
//                     Ok(())
//                 }),
//         )
//         .get_matches();
//     const SEED_SIZE: usize = 32;
//     let input_seed = matches.value_of("seed").unwrap();
//     let mut hasher = VarBlake2b::new(SEED_SIZE).unwrap();
//     hasher.update(input_seed);
//     let mut seed: [u8; SEED_SIZE] = Default::default();
//     hasher.finalize_variable(|hashed_input_seed| {
//         seed.copy_from_slice(hashed_input_seed);
//     });

//     let neighborhood_rng = rand_chacha::ChaCha20Rng::from_seed(seed);
//     let solver_rng = rand_chacha::ChaCha20Rng::from_seed(seed);
//     let board_size = matches
//         .value_of("board_size")
//         .unwrap()
//         .parse::<u64>()
//         .unwrap();

//     let neighborhood = NQueenNeighborhood::new(board_size, neighborhood_rng);
//     let mut solver: LocalSearchSolver<
//         NQueensValue,
//         NQueensDecisionVariable,
//         NQueenSolution,
//         NQueenNeighborhood,
//         rand_chacha::ChaCha20Rng,
//     > = LocalSearchSolver::new(neighborhood, solver_rng);
//     for _ in 0..100_000 {
//         solver.iterate();
//         if solver.get_best_solution().get_hard_score() == 0 {
//             break;
//         }
//     }
//     let solution = solver.get_best_solution();
//     // println!("solution:\n{:?}", solution);
//     println!("solution hard score: {:?}", solution.get_hard_score());
// }

fn main() {
    println!("Hello, world!");
}