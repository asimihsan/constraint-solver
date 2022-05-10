use std::collections::{BTreeSet, HashMap, HashSet};

use chrono::{Datelike, NaiveDate};

use employee_scheduling::{get_solution, Employee, Holiday, MainArgs, ScheduleScore, ScheduleSolution};
use itertools::Itertools;
use local_search::local_search::ScoredSolution;
use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EmployeeSchedulingInput {
    #[serde(rename = "startDate")]
    pub start_date: NaiveDate,

    #[serde(rename = "endDate")]
    pub end_date: NaiveDate,

    pub employees: Vec<Employee>,

    #[serde(rename = "employeeHolidays")]
    pub employee_holidays: Vec<Vec<NaiveDate>>,
}

#[derive(thiserror::Error, Debug)]
pub enum EmployeeSchedulingError {
    #[error("deserializing input failed")]
    DeserializationError,
}

#[derive(Serialize)]
pub struct ScoredSolutionWrapper {
    pub score: ScheduleScore,
    pub days_to_employees: Vec<(NaiveDate, Employee)>,
}

#[wasm_bindgen]
pub fn solve(input: &JsValue) -> JsValue {
    let input: EmployeeSchedulingInput = input.into_serde().unwrap();
    let employee_to_holidays: HashMap<Employee, HashSet<Holiday>> =
        itertools::zip(input.employees.clone(), input.employee_holidays)
            .map(|(employee, holidays)| {
                (
                    employee,
                    HashSet::from_iter(holidays.iter().map(|holiday| Holiday(*holiday))),
                )
            })
            .collect();
    let solution = solve_inner(
        input.start_date,
        input.end_date,
        BTreeSet::from_iter(input.employees.iter().copied()),
        employee_to_holidays,
    );
    let solution_wrapper = ScoredSolutionWrapper {
        score: solution.score,
        days_to_employees: solution.solution.get_days_to_employees(),
    };
    JsValue::from_serde(&solution_wrapper).unwrap()
}

fn solve_inner(
    start_date: NaiveDate,
    end_date: NaiveDate,
    employees: BTreeSet<Employee>,
    employee_to_holidays: HashMap<Employee, HashSet<Holiday>>,
) -> ScoredSolution<ScheduleSolution, ScheduleScore> {
    println!("employee scheduling local search example");
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
