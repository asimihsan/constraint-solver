use blake2::{digest::consts::U32, Blake2b, Digest};
use local_search::iterated_local_search::AcceptanceCriterion;
use local_search::iterated_local_search::IteratedLocalSearch;
use local_search::local_search::History;
use local_search::local_search::LocalSearch;
use nqueens::NQueensInitialSolutionGenerator;
use nqueens::NQueensMoveProposer;
use nqueens::NQueensPerturbation;
use nqueens::NQueensScore;
use nqueens::NQueensSolution;
use nqueens::NQueensSolutionScoreCalculator;
use rand::SeedableRng;

type Blake2b256 = Blake2b<U32>;

fn main() {
    println!("local search n-queens example");
    let matches = clap::App::new("Local Search N-Queens Example")
        .version("1.0")
        .arg(
            clap::Arg::with_name("seed")
                .short("s")
                .long("seed")
                .value_name("STRING")
                .help("Random seeed, any string")
                .required(false)
                .default_value("42")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("board_size")
                .short("b")
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
    let input_seed = matches.value_of("seed").unwrap();
    let mut hasher = Blake2b256::new();
    hasher.update(input_seed.as_bytes());
    let seed = hasher.finalize();
    let board_size = matches.value_of("board_size").unwrap().parse::<u64>().unwrap();

    let local_search_max_iterations = 10_000;
    let window_size = board_size * 5;
    let best_solutions_capacity = 32;
    let all_solutions_capacity = 100_000;
    let all_solution_iteration_expiry = 1_000;
    let move_proposer = NQueensMoveProposer::new(board_size as usize);
    let solution_score_calculator = NQueensSolutionScoreCalculator::default();
    let solver_rng = rand_chacha::ChaCha20Rng::from_seed(seed.into());
    let local_search: LocalSearch<
        rand_chacha::ChaCha20Rng,
        NQueensSolution,
        NQueensScore,
        NQueensSolutionScoreCalculator,
        NQueensMoveProposer,
    > = LocalSearch::new(
        move_proposer,
        solution_score_calculator,
        local_search_max_iterations,
        window_size.try_into().unwrap(),
        best_solutions_capacity,
        all_solutions_capacity,
        all_solution_iteration_expiry,
        solver_rng,
    );

    let initial_solution_generator = NQueensInitialSolutionGenerator::new(board_size as usize);
    let perturbation = NQueensPerturbation::default();
    let history = History::<rand_chacha::ChaCha20Rng, NQueensSolution, NQueensScore>::new(
        best_solutions_capacity,
        all_solutions_capacity,
        all_solution_iteration_expiry,
    );
    let acceptance_criterion = AcceptanceCriterion::default();
    let iterated_local_search_rng = rand_chacha::ChaCha20Rng::from_seed(seed.into());
    let iterated_local_search_max_iterations = 10_000;
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
        local_search,
        perturbation,
        history,
        acceptance_criterion,
        iterated_local_search_max_iterations,
        iterated_local_search_rng,
    );

    let result = iterated_local_search.execute();
    println!("result.solution:\n{:?}", result.solution);
    println!("result.score: {:?}", result.score);
}
