/// local_search contains methods that represent a solution and proposing moves in the neighborhood of a solution.
/// Use methods in this module you can discover local minima. This is the LocalSearch part of [1] section 2pages 2 and
/// 3.
///
/// [1] Lourenço, Helena Ramalhinho, Olivier C. Martin and Thomas Stützle. "Iterated Local Search: Framework and
/// Applications." (2010).

/// Solution is a plain old data object.
pub trait Solution: Clone + Send + PartialEq + Eq + std::hash::Hash + std::fmt::Debug {}

/// Score for a solution. Could just be e.g. u64, f64, num::Num. Could be more complicated like a tuple
/// (hard score, soft score).
pub trait Score: Clone + Send + PartialEq + Eq + PartialOrd + Ord + std::fmt::Debug {}

#[derive(Derivative)]
#[derivative(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScoredSolution<_Solution, _Score>
where
    _Solution: Solution,
    _Score: Score,
{
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Ord = "ignore")]
    pub solution: _Solution,

    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    pub score: _Score,
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
    type Solution: Solution;
    type _Score: Score;

    /// get_score calculates the score of a solution. See SolutionScoreCalculator doc for ideas about what the score
    /// should be.
    fn get_score(&self, solution: &Self::Solution) -> Self::_Score;
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

/// LocalSearch lets you find local minima for an optimization problem.
pub struct LocalSearch<R, _Solution, _Score, SSC, MP>
where
    R: rand::Rng,
    _Solution: Solution,
    _Score: Score,
    SSC: SolutionScoreCalculator<Solution = _Solution, _Score = _Score>,
    MP: MoveProposer<R = R, Solution = _Solution>,
{
    move_proposer: MP,
    solution_score_calculator: SSC,
    max_iterations: u64,
    rng: R,
}

impl<R, _Solution, _Score, SSC, MP> LocalSearch<R, _Solution, _Score, SSC, MP>
where
    R: rand::Rng,
    _Solution: Solution,
    _Score: Score,
    SSC: SolutionScoreCalculator<Solution = _Solution, _Score = _Score>,
    MP: MoveProposer<R = R, Solution = _Solution>,
{
    pub fn new(move_proposer: MP, solution_score_calculator: SSC, max_iterations: u64, rng: R) -> Self {
        LocalSearch {
            move_proposer,
            solution_score_calculator,
            max_iterations,
            rng,
        }
    }

    pub fn execute(&mut self, start: _Solution) -> ScoredSolution<_Solution, _Score> {
        let mut current_solution = start;
        for _current_iteration in 0..self.max_iterations {
            match self
                .move_proposer
                .iter_local_moves(&current_solution, &mut self.rng)
                .into_iter()
                .filter(|proposed_solution| {
                    self.solution_score_calculator.get_score(proposed_solution)
                        < self.solution_score_calculator.get_score(&current_solution)
                })
                .next()
            {
                Some(new_solution) => current_solution = new_solution,
                None => break,
            }
        }
        ScoredSolution::new(
            current_solution.clone(),
            self.solution_score_calculator.get_score(&current_solution),
        )
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
    use float_ord::FloatOrd;
    use rand::SeedableRng;

    #[test]
    fn ackley_local_minima_found() {
        println!("test: ackley_local_minima_found");
        let dimensions = 2;
        let move_size = 0.1;
        let max_iterations = 100_000;
        let seed = 42;
        let move_proposer = AckleyMoveProposer::new(dimensions, move_size);
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
            solver_rng,
        );

        let mut initial_solution_rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
        let start = initial_solution_generator.generate_initial_solution(&mut initial_solution_rng);
        println!("start: {:?}", start);
        let end = local_search.execute(start.clone());

        let solution_score_calculator = AckleySolutionScoreCalculator::default();
        let start_score = solution_score_calculator.get_score(&start);

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
        let move_size = 0.1;
        let max_iterations = 100_000;
        let seed = 42;
        let move_proposer = AckleyMoveProposer::new(dimensions, move_size);
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
            solver_rng,
        );

        let start = AckleySolution::new((0..dimensions).map(|_| FloatOrd(0.0)).collect());
        println!("start: {:?}", start);
        let end = local_search.execute(start.clone());

        let solution_score_calculator = AckleySolutionScoreCalculator::default();
        let start_score = solution_score_calculator.get_score(&start);

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
