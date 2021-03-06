/// iterated_local_search builds upon local_search, see [1] page 7 algorithm 1.
///
/// [1] Lourenço, Helena Ramalhinho, Olivier C. Martin and Thomas Stützle. "Iterated Local Search: Framework and
/// Applications." (2010).
use std::marker::PhantomData;

use crate::local_search::History;
use crate::local_search::InitialSolutionGenerator;
use crate::local_search::LocalSearch;
use crate::local_search::MoveProposer;
use crate::local_search::Score;
use crate::local_search::ScoredSolution;
use crate::local_search::Solution;
use crate::local_search::SolutionScoreCalculator;
use rand::prelude::SliceRandom;
use serde::Serialize;

/// AcceptanceCriterion takes the old local minima and new local minima, combines it with the history, and determines
/// which one to use.
#[derive(Derivative)]
#[derivative(Default)]
pub struct AcceptanceCriterion<_R, _Solution, _Score, _SSC>
where
    _R: rand::Rng,
    _Solution: Solution,
    _Score: Score,
    _SSC: SolutionScoreCalculator,
{
    phantom_r: PhantomData<_R>,
    phantom_solution: PhantomData<_Solution>,
    phantom_score: PhantomData<_Score>,
    phantom_ssc: PhantomData<_SSC>,
}

impl<_R, _Solution, _Score, _SSC> AcceptanceCriterion<_R, _Solution, _Score, _SSC>
where
    _R: rand::Rng,
    _Solution: Solution,
    _Score: Score,
    _SSC: SolutionScoreCalculator,
{
    pub fn new() -> Self {
        Self {
            phantom_r: PhantomData,
            phantom_solution: PhantomData,
            phantom_score: PhantomData,
            phantom_ssc: PhantomData,
        }
    }

    pub fn choose(
        &mut self,
        existing_local_minima: &ScoredSolution<_Solution, _Score>,
        new_local_minima: &ScoredSolution<_Solution, _Score>,
        history: &History<_R, _Solution, _Score>,
        rng: &mut _R,
    ) -> ScoredSolution<_Solution, _Score> {
        // if new_local_minima.score < existing_local_minima.score {
        //     return new_local_minima;
        // }
        let maybe_random_best_solution = history.get_random_best_solution(rng);
        let choices = match maybe_random_best_solution {
            Some(ref random_best_solution) => vec![
                (existing_local_minima, 1),
                (new_local_minima, 5),
                (random_best_solution, 1),
            ],
            None => vec![(existing_local_minima, 1), (new_local_minima, 5)],
        };
        choices.choose_weighted(rng, |item| item.1).unwrap().0.clone()
    }
}

/// Perturbation takes the current local minima and the history and proposes a new starting point for LocalSearch
/// to start from.
pub trait Perturbation {
    type _R: rand::Rng;
    type _Solution: Solution;
    type _Score: Score;
    type _SSC: SolutionScoreCalculator<_Solution = Self::_Solution, _Score = Self::_Score>;

    fn propose_new_starting_solution(
        &mut self,
        current: &ScoredSolution<Self::_Solution, Self::_Score>,
        history: &History<Self::_R, Self::_Solution, Self::_Score>,
        rng: &mut Self::_R,
    ) -> Self::_Solution;
}

#[derive(Serialize)]
pub struct IterationInfo {
    pub current: u64,
    pub total: u64,
}

pub struct IteratedLocalSearch<_R, _Solution, _Score, _SSC, _MP, _ISG, _P>
where
    _R: rand::Rng,
    _Score: Score,
    _Solution: Solution,
    _SSC: SolutionScoreCalculator<_Solution = _Solution, _Score = _Score>,
    _MP: MoveProposer<R = _R, Solution = _Solution>,
    _ISG: InitialSolutionGenerator,
    _P: Perturbation<_R = _R, _Solution = _Solution, _Score = _Score, _SSC = _SSC>,
{
    initial_solution_generator: _ISG,
    solution_score_calculator: _SSC,
    local_search: LocalSearch<_R, _Solution, _Score, _SSC, _MP>,
    perturbation: _P,
    history: History<_R, _Solution, _Score>,
    acceptance_criterion: AcceptanceCriterion<_R, _Solution, _Score, _SSC>,
    iteration: u64,
    max_iterations: u64,
    max_allow_no_improvement_for: u64,
    rng: _R,
    current: ScoredSolution<_Solution, _Score>,
}

impl<_R, _Solution, _Score, _SSC, _MP, _ISG, _P>
    IteratedLocalSearch<_R, _Solution, _Score, _SSC, _MP, _ISG, _P>
where
    _R: rand::Rng,
    _Score: Score,
    _Solution: Solution,
    _SSC: SolutionScoreCalculator<_Solution = _Solution, _Score = _Score>,
    _MP: MoveProposer<R = _R, Solution = _Solution>,
    _ISG: InitialSolutionGenerator<R = _R, Solution = _Solution>,
    _P: Perturbation<_R = _R, _Solution = _Solution, _Score = _Score, _SSC = _SSC>,
{
    pub fn new(
        initial_solution_generator: _ISG,
        solution_score_calculator: _SSC,
        local_search: LocalSearch<_R, _Solution, _Score, _SSC, _MP>,
        perturbation: _P,
        history: History<_R, _Solution, _Score>,
        acceptance_criterion: AcceptanceCriterion<_R, _Solution, _Score, _SSC>,
        max_iterations: u64,
        max_allow_no_improvement_for: u64,
        mut rng: _R,
    ) -> Self {
        let current = solution_score_calculator
            .get_scored_solution(initial_solution_generator.generate_initial_solution(&mut rng));
        IteratedLocalSearch {
            initial_solution_generator,
            solution_score_calculator,
            local_search,
            perturbation,
            history,
            acceptance_criterion,
            iteration: 0,
            max_iterations,
            max_allow_no_improvement_for,
            rng,
            current,
        }
    }

    pub fn get_iteration_info(&self) -> IterationInfo {
        IterationInfo {
            current: self.iteration,
            total: self.max_iterations,
        }
    }

    pub fn get_best_solution(&self) -> ScoredSolution<_Solution, _Score> {
        self.history.get_best().unwrap()
    }

    pub fn is_finished(&self) -> bool {
        self.iteration >= self.max_iterations
    }

    pub fn execute_round(&mut self) {
        self.iteration += 1;
        if let Some(best) = self.history.get_best() {
            println!(
                "iterated local search best score: {:?}, current score {:?}",
                &best.score, &self.current.score
            );
            if best.score.is_best() {
                println!("iterated local search found best possible solution and is terminating");
                return;
            }
        }
        if self.iteration > 0 && self.iteration % 50 == 0 {
            println!("reset from random");
            self.current = self.solution_score_calculator.get_scored_solution(
                self.initial_solution_generator
                    .generate_initial_solution(&mut self.rng),
            );
        }
        let perturbed =
            self.perturbation
                .propose_new_starting_solution(&self.current, &self.history, &mut self.rng);
        let new = self
            .local_search
            .execute(perturbed, self.max_allow_no_improvement_for);
        self.history.local_search_chose_solution(new.clone());
        self.current = self
            .acceptance_criterion
            .choose(&self.current, &new, &self.history, &mut self.rng);
    }
}

#[cfg(test)]
mod ackley_tests {
    use approx::assert_abs_diff_eq;
    use rand::SeedableRng;

    use crate::ackley::AckleyPerturbation;
    use crate::ackley::{
        AckleyInitialSolutionGenerator, AckleyMoveProposer, AckleyScore, AckleySolution,
        AckleySolutionScoreCalculator,
    };
    use crate::iterated_local_search::AcceptanceCriterion;
    use crate::iterated_local_search::History;
    use crate::iterated_local_search::IteratedLocalSearch;
    use crate::local_search::LocalSearch;
    use crate::local_search::ScoredSolution;

    fn _ackley(dimensions: usize, seed: u64) -> ScoredSolution<AckleySolution, AckleyScore> {
        let min_move_size = 1e-3;
        let max_move_size = 0.5;
        let local_search_max_iterations = 100_000;
        let window_size = 500;
        let best_solutions_capacity = 16;
        let all_solutions_capacity = 10_000;
        let all_solution_iteration_expiry = 10_000;
        let move_proposer = AckleyMoveProposer::new(dimensions, min_move_size, max_move_size);
        let solution_score_calculator = AckleySolutionScoreCalculator::default();
        let solver_rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
        let local_search: LocalSearch<
            rand_chacha::ChaCha20Rng,
            AckleySolution,
            AckleyScore,
            AckleySolutionScoreCalculator,
            AckleyMoveProposer,
        > = LocalSearch::new(
            move_proposer,
            solution_score_calculator,
            local_search_max_iterations,
            window_size,
            best_solutions_capacity,
            all_solutions_capacity,
            all_solution_iteration_expiry,
            solver_rng,
        );

        let initial_solution_generator = AckleyInitialSolutionGenerator::new(dimensions);
        let solution_score_calculator = AckleySolutionScoreCalculator::default();
        let perturbation = AckleyPerturbation::default();
        let history = History::<rand_chacha::ChaCha20Rng, AckleySolution, AckleyScore>::default();
        let acceptance_criterion = AcceptanceCriterion::default();
        let iterated_local_search_rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
        let iterated_local_search_max_iterations = 10_000;
        let max_allow_no_improvement_for = 5;
        let mut iterated_local_search: IteratedLocalSearch<
            rand_chacha::ChaCha20Rng,
            AckleySolution,
            AckleyScore,
            AckleySolutionScoreCalculator,
            AckleyMoveProposer,
            AckleyInitialSolutionGenerator,
            AckleyPerturbation,
        > = IteratedLocalSearch::new(
            initial_solution_generator,
            solution_score_calculator,
            local_search,
            perturbation,
            history,
            acceptance_criterion,
            iterated_local_search_max_iterations,
            max_allow_no_improvement_for,
            iterated_local_search_rng,
        );

        while !iterated_local_search.is_finished() {
            iterated_local_search.execute_round();
        }
        iterated_local_search.get_best_solution()
    }

    #[test]
    fn ackley() {
        let dimensions = 2;
        for seed in 0..1 {
            let solution = _ackley(dimensions, seed);
            println!(
                "iterated local search ackley seed {} dimensions {} solution score {:.2}: {:?}",
                seed,
                dimensions,
                solution.score.get_score(),
                solution
            );
            assert_abs_diff_eq!(0.0, solution.score.get_score(), epsilon = 1e-2);
        }

        let dimensions = 10;
        for seed in 0..1 {
            let solution = _ackley(dimensions, seed);
            println!(
                "iterated local search ackley seed {} dimensions {} solution score {:.2}: {:?}",
                seed,
                dimensions,
                solution.score.get_score(),
                solution
            );
            assert_abs_diff_eq!(0.0, solution.score.get_score(), epsilon = 1e-2);
        }

        let dimensions = 20;
        for seed in 0..1 {
            let solution = _ackley(dimensions, seed);
            println!(
                "iterated local search ackley seed {} dimensions {} solution score {:.2}: {:?}",
                seed,
                dimensions,
                solution.score.get_score(),
                solution
            );
            assert_abs_diff_eq!(0.0, solution.score.get_score(), epsilon = 1e-2);
        }
    }
}
