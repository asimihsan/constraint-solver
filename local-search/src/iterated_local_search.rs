use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::local_search::InitialSolutionGenerator;
use crate::local_search::LocalSearch;
use crate::local_search::MoveProposer;
/// iterated_local_search builds upon local_search, see [1] page 7 algorithm 1.
///
/// [1] Lourenço, Helena Ramalhinho, Olivier C. Martin and Thomas Stützle. "Iterated Local Search: Framework and
/// Applications." (2010).
use crate::local_search::{Score, Solution, SolutionScoreCalculator};

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

    pub fn is_empty(&self) -> bool {
        self.best_solutions.is_empty()
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

    pub fn get_best_multiple(&self, number_to_get: usize) -> Option<Vec<S>> {
        if self.best_solutions.is_empty() {
            return None;
        }
        Some(
            self.best_solutions
                .iter()
                .take(number_to_get)
                .unwrap()
                .cloned(),
        )
    }

    pub fn get_best(&self) -> Option<S> {
        if self.best_solutions.is_empty() {
            return None;
        }
        Some(self.best_solutions.iter().next().unwrap().clone())
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

/// History keeps track of the local minima that LocalSearch finds. You can then ask History for the best solutions
/// it's seen so far.
pub struct History<_R, _Solution, _Score, _SSC>
where
    _R: rand::Rng,
    _Solution: Solution,
    _Score: Score,
    _SSC: SolutionScoreCalculator,
{
    best_solutions: BestSolutions<_Solution, _R>,
}

impl<_R, _Solution, _Score, _SSC> History<_R, _Solution, _Score, _SSC>
where
    _R: rand::Rng,
    _Solution: Solution,
    _Score: Score,
    _SSC: SolutionScoreCalculator,
{
    pub fn new(best_solutions: BestSolutions<_Solution, _R>) -> Self {
        History { best_solutions }
    }

    fn local_search_chose_solution(&mut self, solution: &_Solution) {
        self.best_solutions.insert(solution);
    }

    fn get_best_solutions(&mut self, number_of_solutions: usize) -> Option<Vec<_Solution>> {
        self.best_solutions.get_best_multiple(number_of_solutions)
    }

    fn get_best_solution(&mut self) -> Option<_Solution> {
        self.best_solutions.get_best()
    }
}

impl<_R, _Solution, _Score, _SSC> Default for History<_R, _Solution, _Score, _SSC>
where
    _R: rand::Rng,
    _Solution: Solution,
    _Score: Score,
    _SSC: SolutionScoreCalculator,
{
    fn default() -> Self {
        Self::new(Default::default())
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
        current: &Self::_Solution,
        history: History<Self::_R, Self::_Solution, Self::_Score, Self::_SSC>,
    ) -> Self::_Solution;
}

/// AcceptanceCriterion takes the old local minima and new local minima, combines it with the history, and determines
/// which one to use.
pub trait AcceptanceCriterion {
    type _R: rand::Rng;
    type _Solution: Solution;
    type _Score: Score;
    type _SSC: SolutionScoreCalculator<Solution = Self::_Solution, _Score = Self::_Score>;

    fn choose(
        &mut self,
        existing_local_minima: Self::_Solution,
        new_local_minima: Self::_Solution,
        history: &History<Self::_R, Self::_Solution, Self::_Score, Self::_SSC>,
    ) -> Self::_Solution;
}

pub struct IteratedLocalSearch<_R, _Solution, _Score, _SSC, _MP, _ISG, _P, _AC>
where
    _R: rand::Rng,
    _Score: Score,
    _Solution: Solution,
    _SSC: SolutionScoreCalculator<Solution = _Solution, _Score = _Score>,
    _MP: MoveProposer<R = _R, Solution = _Solution>,
    _ISG: InitialSolutionGenerator,
    _P: Perturbation<_Solution = _Solution, _Score = _Score, _SSC = _SSC>,
    _AC: AcceptanceCriterion<_Solution = _Solution, _Score = _Score, _SSC = _SSC>,
{
    initial_solution_generator: _ISG,
    local_search: LocalSearch<_R, _Solution, _Score, _SSC, _MP>,
    peturbation: _P,
    history: History<_R, _Solution, _Score, _SSC>,
    acceptance_criterion: _AC,
    max_iterations: u64,
    rng: _R,
}

impl<_R, _Solution, _Score, _SSC, _MP, _ISG, _P, _AC>
    IteratedLocalSearch<_R, _Solution, _Score, _SSC, _MP, _ISG, _P, _AC>
where
    _R: rand::Rng,
    _Score: Score,
    _Solution: Solution,
    _SSC: SolutionScoreCalculator<Solution = _Solution, _Score = _Score>,
    _MP: MoveProposer<R = _R, Solution = _Solution>,
    _ISG: InitialSolutionGenerator<R = _R, Solution = _Solution>,
    _P: Perturbation<_Solution = _Solution, _Score = _Score, _SSC = _SSC>,
    _AC: AcceptanceCriterion<_Solution = _Solution, _Score = _Score, _SSC = _SSC>,
{
    pub fn new(
        initial_solution_generator: _ISG,
        local_search: LocalSearch<_R, _Solution, _Score, _SSC, _MP>,
        peturbation: _P,
        history: History<_R, _Solution, _Score, _SSC>,
        acceptance_criterion: _AC,
        max_iterations: u64,
        rng: _R,
    ) -> Self {
        IteratedLocalSearch {
            initial_solution_generator,
            local_search,
            peturbation,
            history,
            acceptance_criterion,
            max_iterations,
            rng,
        }
    }

    pub fn execute(&mut self) -> _Solution {
        let initial = self
            .initial_solution_generator
            .generate_initial_solution(&mut self.rng);
        let mut current = self.local_search.execute(initial);
        for _ in 0..self.max_iterations {
            self.history.local_search_chose_solution(&current);
            let perturbed = self
                .peturbation
                .propose_new_starting_solution(&current, &self.history);
            let new = self.local_search.execute(perturbed);
            current = self
                .acceptance_criterion
                .choose(current, new, &self.history);
        }
        self.history.get_best_solution().unwrap()
    }
}
