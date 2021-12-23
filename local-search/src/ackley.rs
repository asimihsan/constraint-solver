/// ackley is a module that contains an implementation of the Ackley function
/// using the local solver framework.
///
/// This is included as src code because I want to use it both for the plain local search
/// but also for iterated local search.
///
/// Ackley Function is defined in [3] from [2].
///
/// [2] Optimization Test Problems: https://www.sfu.ca/~ssurjano/optimization.html
/// [3] Ackley Function: https://www.sfu.ca/~ssurjano/ackley.html
use float_ord::FloatOrd;
use math_util::ackley::AckleyFunction;
use rand::{prelude::SliceRandom, Rng};
use rand_distr::Distribution;

use crate::iterated_local_search::Perturbation;
use crate::local_search::{InitialSolutionGenerator, MoveProposer, Score, Solution, SolutionScoreCalculator};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AckleySolution {
    x: Vec<FloatOrd<f64>>,
}
impl Solution for AckleySolution {}
impl AckleySolution {
    pub fn new(x: Vec<FloatOrd<f64>>) -> Self {
        AckleySolution { x }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AckleyScore(FloatOrd<f64>);
impl Score for AckleyScore {

    /// Although we know the best score is 0.0 let's assume we don't know what the best score is.
    fn is_best(&self) -> bool {
        false
    }
}
impl AckleyScore {
    pub fn get_score(&self) -> f64 {
        self.0 .0
    }
}

pub struct AckleySolutionScoreCalculator {
    ackley_function: math_util::ackley::AckleyFunction,
}

impl AckleySolutionScoreCalculator {
    pub fn new(ackley_function: AckleyFunction) -> Self {
        AckleySolutionScoreCalculator { ackley_function }
    }
}

impl Default for AckleySolutionScoreCalculator {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl SolutionScoreCalculator for AckleySolutionScoreCalculator {
    type _Solution = AckleySolution;
    type _Score = AckleyScore;

    fn get_score(&self, solution: &Self::_Solution) -> Self::_Score {
        let score = self.ackley_function.calculate(&solution.x);
        AckleyScore(FloatOrd(score))
    }
}

pub struct AckleyInitialSolutionGenerator {
    dimensions: usize,
}

impl AckleyInitialSolutionGenerator {
    pub fn new(dimensions: usize) -> Self {
        AckleyInitialSolutionGenerator { dimensions }
    }
}

impl InitialSolutionGenerator for AckleyInitialSolutionGenerator {
    type R = rand_chacha::ChaCha20Rng;
    type Solution = AckleySolution;

    fn generate_initial_solution(&self, rng: &mut Self::R) -> Self::Solution {
        let x_min = -32.768;
        let x_max = 32.768;
        AckleySolution {
            x: (0..self.dimensions)
                .map(|_| FloatOrd(rng.gen_range(x_min..x_max)))
                .collect(),
        }
    }
}

pub struct AckleyMoveProposer {
    dimensions: usize,
    min_move_size: f64,
    max_move_size: f64,
}

impl AckleyMoveProposer {
    pub fn new(dimensions: usize, min_move_size: f64, max_move_size: f64) -> Self {
        AckleyMoveProposer {
            dimensions,
            min_move_size,
            max_move_size
        }
    }
}

impl Default for AckleyMoveProposer {
    fn default() -> Self {
        Self {
            dimensions: 2,
            min_move_size: 1e-6,
            max_move_size: 0.1,
        }
    }
}

impl MoveProposer for AckleyMoveProposer {
    type R = rand_chacha::ChaCha20Rng;
    type Solution = AckleySolution;

    fn iter_local_moves(
        &self,
        start: &Self::Solution,
        rng: &mut Self::R,
    ) -> Box<dyn Iterator<Item = Self::Solution>> {
        enum MoveUpOrDown {
            Up,
            Down,
        }
        struct MoveIterator {
            dimension_schedule: Vec<usize>,
            current_dimension: usize,
            current_move: MoveUpOrDown,
            dimensions: usize,
            move_size: f64,
            start_solution: AckleySolution,
        }
        impl Iterator for MoveIterator {
            type Item = AckleySolution;

            fn next(&mut self) -> Option<Self::Item> {
                if self.current_dimension >= self.dimensions {
                    return None;
                }
                let dimension_from_schedule = self.dimension_schedule[self.current_dimension];
                let mut current_solution = self.start_solution.clone();
                match self.current_move {
                    MoveUpOrDown::Up => {
                        current_solution.x[dimension_from_schedule] =
                            FloatOrd(current_solution.x[dimension_from_schedule].0 + self.move_size);
                        self.current_move = MoveUpOrDown::Down;
                    }
                    MoveUpOrDown::Down => {
                        current_solution.x[dimension_from_schedule] =
                            FloatOrd(current_solution.x[dimension_from_schedule].0 - self.move_size);
                        self.current_dimension += 1;
                        self.current_move = MoveUpOrDown::Up;
                    }
                }
                Some(current_solution)
            }
        }

        let mut dimension_schedule: Vec<usize> = (0..self.dimensions).collect();
        dimension_schedule.shuffle(rng);
        let move_size = rng.gen_range(self.min_move_size..self.max_move_size);
        Box::new(MoveIterator {
            dimension_schedule,
            current_dimension: 0,
            current_move: MoveUpOrDown::Up,
            dimensions: self.dimensions,
            start_solution: start.clone(),
            move_size,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AckleyPerturbationStrategy {
    ChangeSubset,
    DoNothing
}

pub struct AckleyPerturbation {
    strategy: Vec<(AckleyPerturbationStrategy, u64)>,
}

impl AckleyPerturbation {
    pub fn new(strategy: Vec<(AckleyPerturbationStrategy, u64)>) -> Self {
        Self { strategy }
    }
}

impl Default for AckleyPerturbation {
    fn default() -> Self {
        Self { strategy: vec![
            (AckleyPerturbationStrategy::ChangeSubset, 100),
            (AckleyPerturbationStrategy::DoNothing, 10),
        ] }
    }
}

impl Perturbation for AckleyPerturbation {
    type _R = rand_chacha::ChaCha20Rng;
    type _Solution = AckleySolution;
    type _Score = AckleyScore;
    type _SSC = AckleySolutionScoreCalculator;

    fn propose_new_starting_solution(
        &mut self,
        current: &crate::local_search::ScoredSolution<Self::_Solution, Self::_Score>,
        _history: &crate::local_search::History<Self::_R, Self::_Solution, Self::_Score>,
        rng: &mut Self::_R,
    ) -> Self::_Solution {
        let _x_min = -32.768;
        let _x_max = 32.768;
        let current_strategy = self.strategy.choose_weighted(rng, |s| s.1).unwrap().0.clone();
        match current_strategy {
            AckleyPerturbationStrategy::ChangeSubset => {
                let mut new_solution = current.solution.clone();
                let mut dimensions: Vec<usize> = (0..new_solution.x.len()).collect();
                dimensions.shuffle(rng);
                let number_of_dimensions_to_alter = rng.gen_range(0..dimensions.len());
                let dimensions_to_alter: Vec<usize> = dimensions.into_iter().take(number_of_dimensions_to_alter).collect();
                for i in dimensions_to_alter {
                    let normal = rand_distr::Normal::new(new_solution.x[i].0, 1.0).unwrap();
                    let v = normal.sample(rng);
                    new_solution.x[i] = FloatOrd(v)
                }
                // println!("change subset perturbed {:?} to {:?}", &current.solution, &new_solution);
                new_solution
            }
            AckleyPerturbationStrategy::DoNothing => {
                current.solution.clone()
            }
        }
    }
}
