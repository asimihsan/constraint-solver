use std::collections::BTreeSet;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::marker::PhantomData;

use rand::prelude::SliceRandom;

/// local_search contains methods that represent a solution and proposing moves in the neighborhood of a solution.
/// Use methods in this module you can discover local minima. This is the LocalSearch part of [1] section 2pages 2 and
/// 3.
///
/// [1] Lourenço, Helena Ramalhinho, Olivier C. Martin and Thomas Stützle. "Iterated Local Search: Framework and
/// Applications." (2010).

/// Solution is a plain old data object.
pub trait Solution:
    Clone + Send + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash + std::fmt::Debug
{
}

/// Score for a solution. Could just be e.g. u64, f64, num::Num. Could be more complicated like a tuple
/// (hard score, soft score).
pub trait Score: Clone + Send + PartialEq + Eq + PartialOrd + Ord + std::fmt::Debug {
    /// Is this the best possible score. For some problem domains you do not know if there is a best score, so you
    /// can return false.
    fn is_best(&self) -> bool;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScoredSolution<_Solution, _Score>
where
    _Solution: Solution,
    _Score: Score,
{
    pub score: _Score,
    pub solution: _Solution,
}

impl<_Solution, _Score> ScoredSolution<_Solution, _Score>
where
    _Solution: Solution,
    _Score: Score,
{
    pub fn new(solution: _Solution, score: _Score) -> Self {
        Self { solution, score }
    }
}

/// SolutionScoreCalculator calculates the hard and soft score for a given solution. Implementations do not have to be
/// deterministic; some interesting results come out of randomly perturbing the score of solutions for e.g. the
/// Traveling Salesperson Problem (TSP).
///
/// -    A pure satisfaction problem moves from an infeasible configuration and tries to find any feasible
///      solution. Trying to minimize hard score to zero.
/// -    A pure optimization problem always has feasible solutions but move from suboptimal solutions to
///      more optimal solutions. Hard score always zero, trying to minimize soft score to zero.
///      A constraint optimization problem combines both satisfaction and optimization.
pub trait SolutionScoreCalculator {
    type _Solution: Solution;
    type _Score: Score;

    /// get_scored_solution calculates the score of a solution. See SolutionScoreCalculator doc for ideas about what the score
    /// should be.
    fn get_scored_solution(&self, solution: Self::_Solution)
        -> ScoredSolution<Self::_Solution, Self::_Score>;
}

pub trait InitialSolutionGenerator {
    type R: rand::Rng;
    type Solution: Solution;

    /// Generate an initial solution. Does not have to be feasible, i.e. does not have to have a hard score of zero.
    /// However, many local search applications depend on some greedy construction of a feasible initial solution.
    fn generate_initial_solution(&self, rng: &mut Self::R) -> Self::Solution;
}

/// MoveProposer can give you an initial solution, and promises to let one iterate randomly over the neighborhood of
/// solutions.
pub trait MoveProposer {
    type R: rand::Rng;
    type Solution: Solution;

    /// Iterate over the neighborhood of solutions need a start solution randomly. Must be a finite-sized iterator that
    /// is computationally feasbile to fully consume. However, local search will typically not exhaust this iterator.
    fn iter_local_moves(
        &self,
        start: &Self::Solution,
        rng: &mut Self::R,
    ) -> Box<dyn Iterator<Item = Self::Solution>>;
}

#[derive(Derivative)]
#[derivative(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ScoredSolutionAndIterationAdded<_Solution, _Score>
where
    _Solution: Solution,
    _Score: Score,
{
    scored_solution: ScoredSolution<_Solution, _Score>,
    iteration: u64,
}

/// History keeps track of the all solutions that LocalSearch finds. You can then ask History for the best solutions
/// it's seen so far, the tabu set, etc.
pub struct History<_R, _Solution, _Score>
where
    _R: rand::Rng,
    _Solution: Solution,
    _Score: Score,
{
    best_solutions: BTreeSet<ScoredSolution<_Solution, _Score>>,
    best_solutions_capacity: usize,
    all_solutions: VecDeque<ScoredSolutionAndIterationAdded<_Solution, _Score>>,
    all_solutions_capacity: usize,
    all_solutions_lookup: HashSet<_Solution>,
    all_solution_iteration_expiry: u64,
    pub iteration_count: u64,
    phantom_r: PhantomData<_R>,
}

impl<_R, _Solution, _Score> Default for History<_R, _Solution, _Score>
where
    _R: rand::Rng,
    _Solution: Solution,
    _Score: Score,
{
    fn default() -> Self {
        Self::new(16, 10_000, 100_000)
    }
}

impl<_R, _Solution, _Score> History<_R, _Solution, _Score>
where
    _R: rand::Rng,
    _Solution: Solution,
    _Score: Score,
{
    pub fn new(
        best_solutions_capacity: usize,
        all_solutions_capacity: usize,
        all_solution_iteration_expiry: u64,
    ) -> Self {
        History {
            best_solutions: Default::default(),
            best_solutions_capacity,
            all_solutions: VecDeque::with_capacity(all_solutions_capacity),
            all_solutions_capacity,
            all_solutions_lookup: Default::default(),
            all_solution_iteration_expiry,
            iteration_count: 0,
            phantom_r: PhantomData,
        }
    }

    pub fn seen_solution(&mut self, solution: ScoredSolution<_Solution, _Score>) {
        self.iteration_count += 1;
        self._pop_solution_for_age();
        if self.all_solutions_lookup.contains(&solution.solution) {
            return;
        }
        self._add_solution(solution);
    }

    fn _add_solution(&mut self, solution: ScoredSolution<_Solution, _Score>) {
        self._pop_solution_for_size();
        self.all_solutions.push_front(ScoredSolutionAndIterationAdded {
            scored_solution: solution.clone(),
            iteration: self.iteration_count,
        });
        self.all_solutions_lookup.insert(solution.solution.clone());
    }

    fn _pop_solution_for_size(&mut self) {
        while self.all_solutions.len() > self.all_solutions_capacity {
            if let Some(solution) = self.all_solutions.pop_back() {
                self.all_solutions_lookup
                    .remove(&solution.scored_solution.solution);
            }
        }
    }

    fn _pop_solution_for_age(&mut self) {
        loop {
            if let Some(solution) = self.all_solutions.back() {
                let inner_solution = &solution.scored_solution.solution;
                if solution.iteration + self.all_solution_iteration_expiry >= self.iteration_count {
                    self.all_solutions_lookup.remove(inner_solution);
                    self.all_solutions.pop_back();
                    continue;
                }
                break;
            }
            break;
        }
    }

    pub fn is_solution_tabu(&self, solution: &_Solution) -> bool {
        self.all_solutions_lookup.contains(solution)
    }

    pub fn is_best_solution(&self, solution: ScoredSolution<_Solution, _Score>) -> bool {
        self.best_solutions.contains(&solution)
    }

    pub fn local_search_chose_solution(&mut self, solution: ScoredSolution<_Solution, _Score>) {
        if self.best_solutions.len() < self.best_solutions_capacity {
            self.best_solutions.insert(solution.clone());
            return;
        }

        // TODO better heuristic for creating a diverse best solution set even if the candidate solution has a worse
        // score.
        let worst_solution = self.best_solutions.iter().next_back().unwrap().clone();
        if solution.score <= worst_solution.score {
            self.best_solutions.remove(&worst_solution);
            self.best_solutions.insert(solution);
        }
    }

    pub fn get_random_best_solution(&self, rng: &mut _R) -> Option<ScoredSolution<_Solution, _Score>> {
        if self.best_solutions.is_empty() {
            return None;
        }
        let best_solutions_vec: Vec<ScoredSolution<_Solution, _Score>> =
            self.best_solutions.iter().cloned().collect();
        let random_best_solution = best_solutions_vec.choose(rng).unwrap().clone();
        Some(random_best_solution)
    }

    pub fn get_best_multiple(&self, number_to_get: usize) -> Option<Vec<ScoredSolution<_Solution, _Score>>> {
        if self.best_solutions.is_empty() {
            return None;
        }
        let result = self.best_solutions.iter().take(number_to_get).cloned().collect();
        Some(result)
    }

    pub fn get_best(&self) -> Option<ScoredSolution<_Solution, _Score>> {
        if self.best_solutions.is_empty() {
            return None;
        }
        Some(self.best_solutions.iter().next().unwrap().clone())
    }

    pub fn clear(&mut self) {
        self.all_solutions.clear();
        self.all_solutions_lookup.clear();
        self.best_solutions.clear();
    }
}

/// LocalSearch lets you find local minima for an optimization problem.
pub struct LocalSearch<R, _Solution, _Score, SSC, MP>
where
    R: rand::Rng,
    _Solution: Solution,
    _Score: Score,
    SSC: SolutionScoreCalculator<_Solution = _Solution, _Score = _Score>,
    MP: MoveProposer<R = R, Solution = _Solution>,
{
    move_proposer: MP,
    solution_score_calculator: SSC,
    max_iterations: u64,
    window_size: usize,
    history: History<R, _Solution, _Score>,
    rng: R,
}

impl<R, _Solution, _Score, SSC, MP> LocalSearch<R, _Solution, _Score, SSC, MP>
where
    R: rand::Rng,
    _Solution: Solution,
    _Score: Score,
    SSC: SolutionScoreCalculator<_Solution = _Solution, _Score = _Score>,
    MP: MoveProposer<R = R, Solution = _Solution>,
{
    pub fn new(
        move_proposer: MP,
        solution_score_calculator: SSC,
        max_iterations: u64,
        window_size: usize,
        best_solutions_capacity: usize,
        all_solutions_capacity: usize,
        all_solution_iteration_expiry: u64,
        rng: R,
    ) -> Self {
        LocalSearch {
            move_proposer,
            solution_score_calculator,
            max_iterations,
            window_size,
            history: History::new(
                best_solutions_capacity,
                all_solutions_capacity,
                all_solution_iteration_expiry,
            ),
            rng,
        }
    }

    pub fn execute(
        &mut self,
        start: _Solution,
        allow_no_improvement_for: u64,
    ) -> ScoredSolution<_Solution, _Score> {
        let mut current_solution = self.solution_score_calculator.get_scored_solution(start);
        let mut best_solution = current_solution.clone();
        let mut no_improvement_for = 0;
        for _current_iteration in 0..self.max_iterations {
            self.history.seen_solution(current_solution.clone());
            if current_solution.score.is_best() {
                println!("local search found best possible solution and is terminating");
                return current_solution;
            }
            let mut neighborhood: Vec<ScoredSolution<_Solution, _Score>> = self
                .move_proposer
                .iter_local_moves(&current_solution.solution, &mut self.rng)
                .into_iter()
                .filter(|solution| !self.history.is_solution_tabu(solution))
                .map(|solution| self.solution_score_calculator.get_scored_solution(solution))
                .take(self.window_size)
                .collect();
            neighborhood.sort();
            // println!("ls neighborhood size {}, best score {:?}", neighborhood.len(), neighborhood.first());
            if let Some(neighborhood_best) = neighborhood.first() {
                if neighborhood_best.score < current_solution.score {
                    best_solution = neighborhood_best.clone();
                    no_improvement_for = 0;
                } else {
                    no_improvement_for += 1;
                    if no_improvement_for >= allow_no_improvement_for {
                        break;
                    }
                }
                current_solution = neighborhood_best.clone();
            } else {
                break;
            }
        }
        // println!("ls best solution: {:?}", best_solution);
        best_solution
    }
}

/// In order to test local search methods, we take a handful of benchmark functions from [2] and make sure that
/// given an initial solution we can find a lower-cost new solution. We also need to make sure that our searches are
/// deterministic for a given random-number generator (RNG).
///
/// [2] Optimization Test Problems: https://www.sfu.ca/~ssurjano/optimization.html
#[cfg(test)]
mod ackley_tests {
    use crate::{
        ackley::{
            AckleyInitialSolutionGenerator, AckleyMoveProposer, AckleyScore, AckleySolution,
            AckleySolutionScoreCalculator,
        },
        local_search::{InitialSolutionGenerator, LocalSearch, SolutionScoreCalculator},
    };
    use approx::assert_abs_diff_eq;
    use ordered_float::OrderedFloat;
    use rand::SeedableRng;

    #[test]
    fn ackley_local_minima_found() {
        println!("test: ackley_local_minima_found");
        let dimensions = 2;
        let min_move_size = 1e-6;
        let max_move_size = 0.1;
        let max_iterations = 100_000;
        let seed = 42;
        let window_size = 256;
        let best_solutions_capacity = 16;
        let all_solutions_capacity = 10_000;
        let all_solution_iteration_expiry = 10_000;
        let move_proposer = AckleyMoveProposer::new(dimensions, min_move_size, max_move_size);
        let initial_solution_generator = AckleyInitialSolutionGenerator::new(dimensions);
        let solution_score_calculator = AckleySolutionScoreCalculator::default();

        let solver_rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
        let mut local_search: LocalSearch<
            rand_chacha::ChaCha20Rng,
            AckleySolution,
            AckleyScore,
            AckleySolutionScoreCalculator,
            AckleyMoveProposer,
        > = LocalSearch::new(
            move_proposer,
            solution_score_calculator,
            max_iterations,
            window_size,
            best_solutions_capacity,
            all_solutions_capacity,
            all_solution_iteration_expiry,
            solver_rng,
        );

        let mut initial_solution_rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
        let start = initial_solution_generator.generate_initial_solution(&mut initial_solution_rng);
        println!("start: {:?}", start);
        let allow_no_improvement_for = 1;
        let end = local_search.execute(start.clone(), allow_no_improvement_for);

        let solution_score_calculator = AckleySolutionScoreCalculator::default();
        let start_score = solution_score_calculator.get_scored_solution(start.clone()).score;

        println!(
            "start_score: {:?}, end_score: {:?}, start: {:?}, end: {:?}",
            start_score, end.score, start, end
        );
        assert!(
            end.score < start_score,
            "expected end_score to be better than start_score"
        );
        assert_ne!(
            start, end.solution,
            "expected end solution to be different from start solution"
        );
    }

    #[test]
    fn ackley_when_starting_from_global_minima_does_not_move() {
        println!("test: ackley_when_starting_from_global_minima_does_not_move");
        let dimensions = 2;
        let min_move_size = 1e-6;
        let max_move_size = 0.1;
        let max_iterations = 100_000;
        let window_size = 256;
        let best_solutions_capacity = 16;
        let all_solutions_capacity = 10_000;
        let all_solution_iteration_expiry = 10_000;
        let seed = 42;
        let move_proposer = AckleyMoveProposer::new(dimensions, min_move_size, max_move_size);
        let solution_score_calculator = AckleySolutionScoreCalculator::default();

        let solver_rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
        let mut local_search: LocalSearch<
            rand_chacha::ChaCha20Rng,
            AckleySolution,
            AckleyScore,
            AckleySolutionScoreCalculator,
            AckleyMoveProposer,
        > = LocalSearch::new(
            move_proposer,
            solution_score_calculator,
            max_iterations,
            window_size,
            best_solutions_capacity,
            all_solutions_capacity,
            all_solution_iteration_expiry,
            solver_rng,
        );

        let start = AckleySolution::new((0..dimensions).map(|_| OrderedFloat(0.0)).collect());
        println!("start: {:?}", start);
        let allow_no_improvement_for = 1;
        let end = local_search.execute(start.clone(), allow_no_improvement_for);

        let solution_score_calculator = AckleySolutionScoreCalculator::default();
        let start_score = solution_score_calculator.get_scored_solution(start.clone()).score;

        println!(
            "start_score: {:?}, end_score: {:?}, start: {:?}, end: {:?}",
            start_score, end.score, start, end
        );
        assert_abs_diff_eq!(end.score.get_score(), start_score.get_score(), epsilon = 1e-12);
        assert_eq!(
            start, end.solution,
            "expected end solution to be same as start solution"
        );
    }
}
