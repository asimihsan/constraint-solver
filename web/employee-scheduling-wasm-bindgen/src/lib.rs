use std::collections::{BTreeSet, HashMap};

use chrono::Duration;
use chrono::{Datelike, NaiveDate};

use employee_scheduling::{get_solution, Employee, MainArgs, ScheduleScore, ScheduleSolution};
use itertools::Itertools;
use local_search::local_search::ScoredSolution;
use serde_derive::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
pub struct ScoredSolutionWrapper {
    pub solution: ScheduleSolution,
    pub score: ScheduleScore,
}

#[wasm_bindgen]
pub fn solve() -> JsValue {
    let solution = example();
    let solution_wrapper = ScoredSolutionWrapper {
        solution: solution.solution,
        score: solution.score,
    };
    JsValue::from_serde(&solution_wrapper).unwrap()
}

fn example() -> ScoredSolution<ScheduleSolution, ScheduleScore> {
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
    let iterated_local_search_max_iterations = 100;
    let max_allow_no_improvement_for = 20;
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
    println!("---");
    for (employee, days) in result.solution.get_employees_to_days().iter().sorted() {
        println!("employee: {:?}", employee);
        for date in days {
            println!("{:?} - {:?}", date.weekday(), date);
        }
        println!("---");
    }
    result
}
