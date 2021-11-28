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
impl Score for AckleyScore {}
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
    type Solution = AckleySolution;
    type _Score = AckleyScore;

    fn get_score(&self, solution: &Self::Solution) -> Self::_Score {
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
    move_size: f64,
}

impl AckleyMoveProposer {
    pub fn new(dimensions: usize, move_size: f64) -> Self {
        AckleyMoveProposer {
            dimensions,
            move_size,
        }
    }
}

impl Default for AckleyMoveProposer {
    fn default() -> Self {
        Self {
            dimensions: 2,
            move_size: 0.01,
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

        Box::new(MoveIterator {
            dimension_schedule,
            current_dimension: 0,
            current_move: MoveUpOrDown::Up,
            dimensions: self.dimensions,
            start_solution: start.clone(),
            move_size: self.move_size,
        })
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct AckleyPerturbation {}

impl AckleyPerturbation {
    pub fn new() -> Self {
        Self {}
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
        history: &crate::iterated_local_search::History<Self::_R, Self::_Solution, Self::_Score>,
        rng: &mut Self::_R,
    ) -> Self::_Solution {
        // HACK full restart every 1000 iterations
        if history.iteration_count % 100 == 0 {
            let x_min = -32.768;
            let x_max = 32.768;
            AckleySolution {
                x: (0..current.solution.x.len())
                    .map(|_| FloatOrd(rng.gen_range(x_min..x_max)))
                    .collect(),
            }
        } else {
            current.solution.clone()
        }
    }
}
