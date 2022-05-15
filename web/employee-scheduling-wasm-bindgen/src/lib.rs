// Reference:
//
// "Opaque Pointer" pattern: https://github.com/rustwasm/wasm-bindgen/issues/1242

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use employee_scheduling::{get_ils, Employee, Holiday, IlsType, MainArgs, ScheduleScore};

#[wasm_bindgen]
pub struct SolverContext {
    solver: IlsType,
}

#[wasm_bindgen]
pub fn create_solver(input: &JsValue) -> SolverContext {
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
    let seed = "42";
    let local_search_max_iterations = 1_000;
    let window_size = 100;
    let best_solutions_capacity = 64;
    let all_solutions_capacity = 100_000;
    let all_solution_iteration_expiry = 1_000;
    let iterated_local_search_max_iterations = 250;
    let max_allow_no_improvement_for = 20;
    let ils = get_ils(MainArgs {
        start_date: input.start_date,
        end_date: input.end_date,
        employees: input.employees.iter().copied().collect(),
        employee_to_holidays,
        seed,
        local_search_max_iterations,
        window_size,
        best_solutions_capacity,
        all_solutions_capacity: all_solutions_capacity as usize,
        all_solution_iteration_expiry,
        iterated_local_search_max_iterations,
        max_allow_no_improvement_for,
    });
    SolverContext { solver: ils }
}

#[wasm_bindgen]
pub fn execute_solver_round(ctx: &mut SolverContext) {
    ctx.solver.execute_round();
}

#[wasm_bindgen]
pub fn get_iteration_info(ctx: &mut SolverContext) -> JsValue {
    let result = ctx.solver.get_iteration_info();
    JsValue::from_serde(&result).unwrap()
}

#[wasm_bindgen]
pub fn is_solver_finished(ctx: &SolverContext) -> bool {
    ctx.solver.is_finished()
}

#[wasm_bindgen]
pub fn get_best_solution(ctx: &SolverContext) -> JsValue {
    let solution = ctx.solver.get_best_solution();
    let solution_wrapper = ScoredSolutionWrapper {
        score: solution.score,
        days_to_employees: solution
            .solution
            .get_days_to_employees()
            .into_iter()
            .map(|(day, employee)| (day.format("%a %Y-%m-%d").to_string(), employee))
            .collect(),
    };
    JsValue::from_serde(&solution_wrapper).unwrap()
}

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
    pub days_to_employees: Vec<(String, Employee)>,
}
