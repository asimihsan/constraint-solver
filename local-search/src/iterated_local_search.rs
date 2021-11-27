/// iterated_local_search builds upon local_search, see [1] page 7 algorithm 1.
///
/// [1] Lourenço, Helena Ramalhinho, Olivier C. Martin and Thomas Stützle. "Iterated Local Search: Framework and
/// Applications." (2010).
use crate::local_search::{Score, Solution, SolutionScoreCalculator};

/// History keeps track of the local minima that LocalSearch finds. You can then ask History for the best solutions
/// it's seen so far.
pub trait History {
    type _Solution: Solution;
    type _Score: Score;
    type _SSC: SolutionScoreCalculator<Solution = Self::_Solution, _Score = Self::_Score>;

    fn local_search_chose_solution(&mut self, solution: &Self::_Solution);
    fn get_best_solutions(&mut self, number_of_solutions: usize) -> Option<Vec<Self::_Solution>>;

    fn get_best_solution(&mut self) -> Option<Self::_Solution> {
        match self.get_best_solutions(1) {
            Some(solutions) => match solutions.first() {
                Some(solution) => Some(solution.clone()),
                None => None,
            },
            None => None,
        }
    }
}

/// Perturbation takes the current local minima and the history and proposes a new starting point for LocalSearch
/// to start from.
pub trait Perturbation {
    type _Solution: Solution;
    type _Score: Score;
    type _SSC: SolutionScoreCalculator<Solution = Self::_Solution, _Score = Self::_Score>;
    type _History: History<_Solution = Self::_Solution, _Score = Self::_Score, _SSC = Self::_SSC>;

    fn propose_new_starting_solution(
        &mut self,
        current: &Self::_Solution,
        history: &Self::_History,
    ) -> Self::_Solution;
}

/// AcceptanceCriterion takes the old local minima and new local minima, combines it with the history, and determines
/// which one to use.
pub trait AcceptanceCriterion {
    type _Solution: Solution;
    type _Score: Score;
    type _SSC: SolutionScoreCalculator<Solution = Self::_Solution, _Score = Self::_Score>;
    type _History: History<_Solution = Self::_Solution, _Score = Self::_Score, _SSC = Self::_SSC>;

    fn choose_local_minima(
        &mut self,
        existing_local_minima: Self::_Solution,
        new_local_minima: Self::_Solution,
        history: &Self::_History,
    ) -> Self::_Solution;
}
