/// iterated_local_search builds upon local_search, see [1] page 7 algorithm 1.
///
/// [1] Lourenço, Helena Ramalhinho, Olivier C. Martin and Thomas Stützle. "Iterated Local Search: Framework and
/// Applications." (2010).
use rand::prelude::SliceRandom;
use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::local_search::InitialSolutionGenerator;
use crate::local_search::LocalSearch;
use crate::local_search::MoveProposer;
use crate::local_search::Score;
use crate::local_search::ScoredSolution;
use crate::local_search::Solution;
use crate::local_search::SolutionScoreCalculator;

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
        existing_local_minima: ScoredSolution<_Solution, _Score>,
        new_local_minima: ScoredSolution<_Solution, _Score>,
        history: &History<_R, _Solution, _Score>,
        rng: &mut _R,
    ) -> ScoredSolution<_Solution, _Score> {
        if new_local_minima.score < existing_local_minima.score {
            return new_local_minima;
        }
        let choices = match history.get_random_best_solution(rng) {
            Some(random_best_solution) => vec![existing_local_minima, new_local_minima, random_best_solution],
            None => vec![existing_local_minima, new_local_minima],
        };
        choices.choose(rng).unwrap().clone()
    }
}

/// History keeps track of the local minima that LocalSearch finds. You can then ask History for the best solutions
/// it's seen so far.
pub struct History<_R, _Solution, _Score>
where
    _R: rand::Rng,
    _Solution: Solution,
    _Score: Score,
{
    best_solutions: BTreeSet<ScoredSolution<_Solution, _Score>>,
    best_solutions_capacity: usize,
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
        Self::new(16, 0)
    }
}

impl<_R, _Solution, _Score> History<_R, _Solution, _Score>
where
    _R: rand::Rng,
    _Solution: Solution,
    _Score: Score,
{
    pub fn new(best_solutions_capacity: usize, iteration_count: u64) -> Self {
        History {
            best_solutions: Default::default(),
            best_solutions_capacity,
            iteration_count,
            phantom_r: PhantomData,
        }
    }

    fn local_search_chose_solution(&mut self, solution: &ScoredSolution<_Solution, _Score>) {
        self.iteration_count += 1;

        if self.best_solutions.len() < self.best_solutions_capacity {
            self.best_solutions.insert(solution.clone());
            return;
        }

        // TODO better heuristic for creating a diverse best solution set even if the candidate solution has a worse
        // score.
        let worst_solution = self.best_solutions.iter().next_back().unwrap().clone();
        if solution.score < worst_solution.score {
            self.best_solutions.remove(&worst_solution);
            self.best_solutions.insert(solution.clone());
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

    pub fn _clear(&mut self) {
        self.best_solutions.clear();
    }
}

/// Perturbation takes the current local minima and the history and proposes a new starting point for LocalSearch
/// to start from.
pub trait Perturbation {
    type _R: rand::Rng;
    type _Solution: Solution;
    type _Score: Score;
    type _SSC: SolutionScoreCalculator<Solution = Self::_Solution, _Score = Self::_Score>;

    fn propose_new_starting_solution(
        &mut self,
        current: &ScoredSolution<Self::_Solution, Self::_Score>,
        history: &History<Self::_R, Self::_Solution, Self::_Score>,
        rng: &mut Self::_R,
    ) -> Self::_Solution;
}

pub struct IteratedLocalSearch<_R, _Solution, _Score, _SSC, _MP, _ISG, _P>
where
    _R: rand::Rng,
    _Score: Score,
    _Solution: Solution,
    _SSC: SolutionScoreCalculator<Solution = _Solution, _Score = _Score>,
    _MP: MoveProposer<R = _R, Solution = _Solution>,
    _ISG: InitialSolutionGenerator,
    _P: Perturbation<_R = _R, _Solution = _Solution, _Score = _Score, _SSC = _SSC>,
{
    initial_solution_generator: _ISG,
    local_search: LocalSearch<_R, _Solution, _Score, _SSC, _MP>,
    perturbation: _P,
    history: History<_R, _Solution, _Score>,
    acceptance_criterion: AcceptanceCriterion<_R, _Solution, _Score, _SSC>,
    max_iterations: u64,
    rng: _R,
}

impl<_R, _Solution, _Score, _SSC, _MP, _ISG, _P>
    IteratedLocalSearch<_R, _Solution, _Score, _SSC, _MP, _ISG, _P>
where
    _R: rand::Rng,
    _Score: Score,
    _Solution: Solution,
    _SSC: SolutionScoreCalculator<Solution = _Solution, _Score = _Score>,
    _MP: MoveProposer<R = _R, Solution = _Solution>,
    _ISG: InitialSolutionGenerator<R = _R, Solution = _Solution>,
    _P: Perturbation<_R = _R, _Solution = _Solution, _Score = _Score, _SSC = _SSC>,
{
    pub fn new(
        initial_solution_generator: _ISG,
        local_search: LocalSearch<_R, _Solution, _Score, _SSC, _MP>,
        perturbation: _P,
        history: History<_R, _Solution, _Score>,
        acceptance_criterion: AcceptanceCriterion<_R, _Solution, _Score, _SSC>,
        max_iterations: u64,
        rng: _R,
    ) -> Self {
        IteratedLocalSearch {
            initial_solution_generator,
            local_search,
            perturbation,
            history,
            acceptance_criterion,
            max_iterations,
            rng,
        }
    }

    pub fn execute(&mut self) -> ScoredSolution<_Solution, _Score> {
        let initial = self
            .initial_solution_generator
            .generate_initial_solution(&mut self.rng);
        let mut current = self.local_search.execute(initial);
        for i in 0..self.max_iterations {
            // if i % 10_000 == 0 {
            //     println!("iterated local search current: {:?}", &current);
            // }
            self.history.local_search_chose_solution(&current);
            let perturbed =
                self.perturbation
                    .propose_new_starting_solution(&current, &self.history, &mut self.rng);
            let new = self.local_search.execute(perturbed);
            current = self
                .acceptance_criterion
                .choose(current, new, &self.history, &mut self.rng);
        }
        self.history.get_best().unwrap()
    }
}

#[cfg(test)]
mod ackley_tests {
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
    use approx::assert_abs_diff_eq;
    use float_ord::FloatOrd;
    use rand::SeedableRng;

    fn _ackley(dimensions: usize, seed: u64) -> ScoredSolution<AckleySolution, AckleyScore> {
        println!("test: ackley");
        let move_size = 0.01;
        let local_search_max_iterations = 100_000;
        let move_proposer = AckleyMoveProposer::new(dimensions, move_size);
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
            solver_rng,
        );

        let initial_solution_generator = AckleyInitialSolutionGenerator::new(dimensions);
        let perturbation = AckleyPerturbation::default();
        let history = History::<rand_chacha::ChaCha20Rng, AckleySolution, AckleyScore>::default();
        let acceptance_criterion = AcceptanceCriterion::default();
        let iterated_local_search_rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
        let iterated_local_search_max_iterations = 10_000;
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
            local_search,
            perturbation,
            history,
            acceptance_criterion,
            iterated_local_search_max_iterations,
            iterated_local_search_rng,
        );

        return iterated_local_search.execute();
    }

    #[test]
    fn ackley() {
        let dimensions = 2;
        for seed in 0..10 {
            let solution = _ackley(dimensions, seed);
            println!("iterated local search ackley seed {} solution score {:.2}: {:?}", seed, solution.score.get_score(), solution);
        }

        let dimensions = 10;
        for seed in 0..2 {
            let solution = _ackley(dimensions, seed);
            println!("iterated local search ackley seed {} solution score {:.2}: {:?}", seed, solution.score.get_score(), solution);
        }

        let dimensions = 20;
        for seed in 0..2 {
            let solution = _ackley(dimensions, seed);
            println!("iterated local search ackley seed {} solution score {:.2}: {:?}", seed, solution.score.get_score(), solution);
        }
    }
}
