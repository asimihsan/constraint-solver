use blake2::{digest::consts::U32, Blake2b, Digest};
use local_search::iterated_local_search::AcceptanceCriterion;
use local_search::iterated_local_search::IteratedLocalSearch;
use local_search::local_search::LocalSearch;
use local_search::local_search::{History, ScoredSolution};
use nqueens::NQueensInitialSolutionGenerator;
use nqueens::NQueensMoveProposer;
use nqueens::NQueensPerturbation;
use nqueens::NQueensScore;
use nqueens::NQueensSolution;
use nqueens::NQueensSolutionScoreCalculator;
use rand::SeedableRng;

type Blake2b256 = Blake2b<U32>;

struct MainArgs<'a> {
    board_size: u64,
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

fn get_solution(args: MainArgs) -> ScoredSolution<NQueensSolution, NQueensScore> {
    let seed = hash_str(args.seed);
    let move_proposer = NQueensMoveProposer::new(args.board_size as usize);
    let solution_score_calculator = NQueensSolutionScoreCalculator::default();
    let solver_rng = rand_chacha::ChaCha20Rng::from_seed(seed);
    let local_search: LocalSearch<
        rand_chacha::ChaCha20Rng,
        NQueensSolution,
        NQueensScore,
        NQueensSolutionScoreCalculator,
        NQueensMoveProposer,
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

    let initial_solution_generator = NQueensInitialSolutionGenerator::new(args.board_size as usize);
    let solution_score_calculator = NQueensSolutionScoreCalculator::default();
    let perturbation = NQueensPerturbation::default();
    let history = History::<rand_chacha::ChaCha20Rng, NQueensSolution, NQueensScore>::new(
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
        NQueensSolution,
        NQueensScore,
        NQueensSolutionScoreCalculator,
        NQueensMoveProposer,
        NQueensInitialSolutionGenerator,
        NQueensPerturbation,
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

fn main() {
    println!("local search n-queens example");
    let matches = clap::App::new("Local Search N-Queens Example")
        .version("1.0")
        .arg(
            clap::Arg::with_name("seed")
                .short('s')
                .long("seed")
                .value_name("STRING")
                .help("Random seeed, any string")
                .required(false)
                .default_value("42")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("board_size")
                .short('b')
                .long("board-size")
                .value_name("INT")
                .help("Board size")
                .required(false)
                .default_value("8")
                .takes_value(true)
                .validator(|input| {
                    if let Err(err) = input.parse::<u64>() {
                        return Err(err.to_string());
                    }
                    Ok(())
                }),
        )
        .get_matches();

    let seed = matches.value_of("seed").unwrap();
    let board_size = matches.value_of("board_size").unwrap().parse::<u64>().unwrap();
    let local_search_max_iterations = 10_000;
    let window_size = board_size * 5;
    let best_solutions_capacity = 32;
    let all_solutions_capacity = 100_000;
    let all_solution_iteration_expiry = 10_000;
    let iterated_local_search_max_iterations = 10_000;
    let max_allow_no_improvement_for = 5;
    let result = get_solution(MainArgs {
        board_size,
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

#[cfg(test)]
mod nqueens_example_tests {
    use super::*;

    #[test]
    fn repeatable() {
        let board_size = 8;
        let local_search_max_iterations = 10_000;
        let window_size = board_size * 5;
        let best_solutions_capacity = 32;
        let all_solutions_capacity = 100_000;
        let all_solution_iteration_expiry = 1_000;
        let iterated_local_search_max_iterations = 10_000;
        let max_allow_no_improvement_for = 5;

        for seed in (42..50).map(|seed| seed.to_string()) {
            let results: Vec<_> = (0..10)
                .map(|i| {
                    println!("repeatable seed: {} i: {}", seed, i);
                    get_solution(MainArgs {
                        board_size,
                        seed: seed.as_str(),
                        local_search_max_iterations,
                        window_size,
                        best_solutions_capacity,
                        all_solutions_capacity,
                        all_solution_iteration_expiry,
                        iterated_local_search_max_iterations,
                        max_allow_no_improvement_for,
                    })
                })
                .collect();

            let (first, rest) = results.split_first().unwrap();
            for other_result in rest.iter() {
                assert_eq!(
                    first, other_result,
                    "two nqueens solutions unexpectedly different with same seed {}",
                    seed
                );
            }

            assert_eq!(
                0, first.score.0,
                "nqueen solution unexpectedly unsatisfiable with seed {}",
                seed
            );
        }
    }
}
