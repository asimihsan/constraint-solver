use std::collections::{BTreeSet, HashMap, HashSet};

use chrono::{Datelike, Duration, NaiveDate};
use itertools::Itertools;

use employee_scheduling::{get_ils, Employee, MainArgs};

fn main() {
    println!("employee scheduling local search example");

    let start_date = NaiveDate::parse_from_str("2022-05-09", "%Y-%m-%d").unwrap();
    let end_date = start_date + Duration::days(30);
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
    let local_search_max_iterations = 1_000;
    let window_size = 100;
    let best_solutions_capacity = 64;
    let all_solutions_capacity = 100_000;
    let all_solution_iteration_expiry = 1_000;
    let iterated_local_search_max_iterations = 250;
    let max_allow_no_improvement_for = 20;

    let mut iterated_local_search = get_ils(MainArgs {
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

    while !iterated_local_search.is_finished() {
        iterated_local_search.execute_round();
    }
    let result = iterated_local_search.get_best_solution();

    println!("result.solution:\n{:?}", result.solution);
    println!("result.score: {:?}", result.score);
    println!("---");
    for (employee, days) in result.solution.get_employees_to_days().iter().sorted() {
        println!("employee: {:?}", employee);
        for date in days {
            println!("{:?} - {:?}", date.weekday(), date);
        }
        println!("---");
    }
}
