use std::collections::{BTreeSet, HashMap, HashSet};

use blake2::{digest::consts::U32, Blake2b, Digest};
use chrono::{Duration, NaiveDate};
use rand::SeedableRng;

use employee_scheduling::{
    Employee, Holiday, ScheduleInitialSolutionGenerator, ScheduleMoveProposer, SchedulePerturbation,
    ScheduleScore, ScheduleSolution, ScheduleSolutionScoreCalculator,
};
use local_search::iterated_local_search::{AcceptanceCriterion, IteratedLocalSearch};
use local_search::local_search::{History, LocalSearch, ScoredSolution};

type Blake2b256 = Blake2b<U32>;

struct MainArgs<'a> {
    start_date: NaiveDate,
    end_date: NaiveDate,
    employees: BTreeSet<Employee>,
    employee_to_holidays: HashMap<Employee, HashSet<Holiday>>,
    seed: &'a str,
    local_search_max_iterations: u64,
    window_size: u64,
    best_solutions_capacity: usize,
    all_solutions_capacity: usize,
    all_solution_iteration_expiry: u64,
    iterated_local_search_max_iterations: u64,
    max_allow_no_improvement_for: u64,
}

fn hash_str(input: &str) -> [u8; 32] {
    let mut hasher = Blake2b256::new();
    hasher.update(input.as_bytes());
    let seed = hasher.finalize();
    seed.into()
}

fn get_solution(args: MainArgs) -> ScoredSolution<ScheduleSolution, ScheduleScore> {
    let seed = hash_str(args.seed);
    let move_proposer = ScheduleMoveProposer::new(args.employees.clone());
    let solution_score_calculator = ScheduleSolutionScoreCalculator::new(args.employee_to_holidays.clone());
    let solver_rng = rand_chacha::ChaCha20Rng::from_seed(seed);
    let local_search: LocalSearch<
        rand_chacha::ChaCha20Rng,
        ScheduleSolution,
        ScheduleScore,
        ScheduleSolutionScoreCalculator,
        ScheduleMoveProposer,
    > = LocalSearch::new(
        move_proposer,
        solution_score_calculator,
        args.local_search_max_iterations,
        args.window_size.try_into().unwrap(),
        args.best_solutions_capacity,
        args.all_solutions_capacity,
        args.all_solution_iteration_expiry,
        solver_rng,
    );

    let initial_solution_generator = ScheduleInitialSolutionGenerator::new(
        args.start_date,
        args.end_date,
        args.employees.clone().iter().copied().collect(),
        args.employee_to_holidays.clone(),
    );
    let solution_score_calculator = ScheduleSolutionScoreCalculator::new(args.employee_to_holidays.clone());
    let perturbation = SchedulePerturbation::default();
    let history = History::<rand_chacha::ChaCha20Rng, ScheduleSolution, ScheduleScore>::new(
        args.best_solutions_capacity,
        args.all_solutions_capacity,
        args.all_solution_iteration_expiry,
    );
    let acceptance_criterion = AcceptanceCriterion::default();
    let iterated_local_search_rng = rand_chacha::ChaCha20Rng::from_seed(seed);
    let iterated_local_search_max_iterations = args.iterated_local_search_max_iterations;
    let max_allow_no_improvement_for = args.max_allow_no_improvement_for;
    let mut iterated_local_search: IteratedLocalSearch<
        rand_chacha::ChaCha20Rng,
        ScheduleSolution,
        ScheduleScore,
        ScheduleSolutionScoreCalculator,
        ScheduleMoveProposer,
        ScheduleInitialSolutionGenerator,
        SchedulePerturbation,
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

    let result = iterated_local_search.execute();
    result
}

fn main() {
    println!("employee scheduling local search example");

    let start_date = NaiveDate::parse_from_str("2022-04-30", "%Y-%m-%d").unwrap();
    let end_date = start_date + Duration::days(60);
    let employees = BTreeSet::from([
        Employee { id: 0 },
        Employee { id: 1 },
        Employee { id: 2 },
        Employee { id: 3 },
        Employee { id: 4 },
        Employee { id: 5 },
        Employee { id: 6 },
    ]);
    let employee_to_holidays = HashMap::new();

    let seed = "42";
    let local_search_max_iterations = 10_000;
    let window_size = 1000;
    let best_solutions_capacity = 32;
    let all_solutions_capacity = 100_000;
    let all_solution_iteration_expiry = 10_000;
    let iterated_local_search_max_iterations = 1_000;
    let max_allow_no_improvement_for = 10;
    let result = get_solution(MainArgs {
        start_date,
        end_date,
        employees,
        employee_to_holidays,
        seed,
        local_search_max_iterations,
        window_size,
        best_solutions_capacity,
        all_solutions_capacity,
        all_solution_iteration_expiry,
        iterated_local_search_max_iterations,
        max_allow_no_improvement_for,
    });

    println!("result.solution:\n{:?}", result.solution);
    println!("result.score: {:?}", result.score);
}
