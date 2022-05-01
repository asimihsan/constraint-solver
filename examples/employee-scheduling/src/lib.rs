#[macro_use]
extern crate derivative;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::ops::Bound::{Excluded, Unbounded};

use chrono::{Datelike, NaiveDate, Weekday};
use itertools::{Itertools, MinMaxResult};
use ordered_float::OrderedFloat;
use rand::prelude::SliceRandom;
use rand::Rng;

use local_search::iterated_local_search::Perturbation;
use local_search::local_search::{
    History, InitialSolutionGenerator, MoveProposer, Score, ScoredSolution, Solution, SolutionScoreCalculator,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Employee {
    pub id: i64,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Holiday(NaiveDate);

#[derive(Derivative)]
#[derivative(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScheduleSolution {
    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Hash = "ignore")]
    start_date: NaiveDate,

    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Hash = "ignore")]
    end_date: NaiveDate,

    pub date_to_employee: Vec<Employee>,

    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Hash = "ignore")]
    pub employees: Vec<Employee>,
}

impl ScheduleSolution {
    pub fn get_employee_for_date(&self, date: NaiveDate) -> Option<Employee> {
        if date < self.start_date || date > self.end_date {
            return None;
        }
        let days_diff = date.signed_duration_since(self.start_date);
        let index = days_diff.num_days() as usize;
        Some(self.date_to_employee[index])
    }

    pub fn get_employees_to_days(&self) -> HashMap<Employee, Vec<NaiveDate>> {
        let mut result = HashMap::with_capacity(self.employees.len());
        for (date, employee) in self.get_days_to_employees() {
            result
                .entry(employee)
                .or_insert_with(|| Vec::with_capacity(self.date_to_employee.len()))
                .push(date);
        }
        result
    }

    pub fn get_days_to_employees(&self) -> Vec<(NaiveDate, Employee)> {
        let mut result = Vec::with_capacity(self.date_to_employee.len());
        for (index, current_date) in self.start_date.iter_days().enumerate() {
            let employee = self.date_to_employee[index];
            result.push((current_date, employee));
            if current_date > self.end_date {
                break;
            }
        }
        result
    }
}

impl Debug for ScheduleSolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        for (date, employee) in self.get_days_to_employees() {
            output += &format!("{} {:?} - {:?}", date.weekday(), date, employee);
            if date <= self.end_date {
                output += "\n";
            }
        }
        f.write_fmt(format_args!("{}", output))
    }
}

impl Solution for ScheduleSolution {}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ScheduleScore {
    pub hard_score: OrderedFloat<f64>,
    pub soft_score: OrderedFloat<f64>,
}

impl Score for ScheduleScore {
    fn is_best(&self) -> bool {
        self.hard_score == 0.0 && self.soft_score == 0.0
    }
}

pub struct ScheduleSolutionScoreCalculator {
    employee_to_holidays: HashMap<Employee, HashSet<Holiday>>,
}

impl ScheduleSolutionScoreCalculator {
    pub fn new(employee_to_holidays: HashMap<Employee, HashSet<Holiday>>) -> Self {
        Self { employee_to_holidays }
    }
}

impl SolutionScoreCalculator for ScheduleSolutionScoreCalculator {
    type _Solution = ScheduleSolution;
    type _Score = ScheduleScore;

    fn get_scored_solution(
        &self,
        solution: Self::_Solution,
    ) -> ScoredSolution<Self::_Solution, Self::_Score> {
        let mut hard_score = 0.0;
        let mut soft_score = 0.0;

        // Holidays are a hard constraint.
        for (employee, holidays) in &self.employee_to_holidays {
            for holiday in holidays {
                let actual_employee = solution.get_employee_for_date(holiday.0).unwrap();
                if actual_employee == *employee {
                    hard_score += 1.0;
                }
            }
        }

        let days_to_employees: Vec<(NaiveDate, Employee)> = solution.get_days_to_employees();
        let employees_to_days = solution.get_employees_to_days();

        // Employee not scheduled on two consecutive days hard constraint.
        for window in days_to_employees.windows(2) {
            let first_employee = window[0].1;
            let second_employee = window[1].1;
            if first_employee == second_employee {
                hard_score += 1.0;
            }
        }

        // Hard constraint, no more than 3 times per 14 days.
        for window in days_to_employees.windows(14) {
            let violations = window
                .iter()
                .map(|(day, employee)| employee)
                .counts()
                .into_iter()
                .filter(|(_employee, count)| *count > 3)
                .count();
            hard_score += violations as f64;
        }

        // Soft constraint, try to schedule employees on same weekdays
        for days in employees_to_days.values() {
            let counts = days
                .iter()
                .filter(|day| day.weekday() != Weekday::Sat && day.weekday() != Weekday::Sun)
                .counts();
            if counts.is_empty() {
                continue;
            }
            let max_count = counts.iter().max_by_key(|elem| *elem.1).unwrap();
            for (date, count) in &counts {
                if **date != **max_count.0 {
                    continue;
                }
                soft_score += *count as f64;
            }
        }

        // Difference in total days is a soft constraint.
        let min_max_days = employees_to_days
            .iter()
            .map(|(_employee, days)| days.len())
            .minmax();
        match min_max_days {
            MinMaxResult::MinMax(min, max) => soft_score += (max - min) as f64,
            _ => {}
        }

        // Difference in total weekends is a soft constraint.
        let min_max_weekends = employees_to_days
            .iter()
            .map(|(_employee, days)| {
                days.into_iter()
                    .filter(|day| day.weekday() == Weekday::Sat || day.weekday() == Weekday::Sun)
                    .collect()
            })
            .map(|days: Vec<&NaiveDate>| days.len())
            .minmax();
        match min_max_weekends {
            MinMaxResult::MinMax(min, max) => soft_score += (max - min) as f64,
            _ => {}
        }

        ScoredSolution {
            score: ScheduleScore {
                hard_score: OrderedFloat(hard_score),
                soft_score: OrderedFloat(soft_score),
            },
            solution,
        }
    }
}

pub struct ScheduleInitialSolutionGenerator {
    start_date: NaiveDate,
    end_date: NaiveDate,
    employees: Vec<Employee>,
    employee_to_holidays: HashMap<Employee, HashSet<Holiday>>,
}

impl ScheduleInitialSolutionGenerator {
    pub fn new(
        start_date: NaiveDate,
        end_date: NaiveDate,
        employees: Vec<Employee>,
        employee_to_holidays: HashMap<Employee, HashSet<Holiday>>,
    ) -> Self {
        Self {
            start_date,
            end_date,
            employees,
            employee_to_holidays,
        }
    }
}

impl InitialSolutionGenerator for ScheduleInitialSolutionGenerator {
    type R = rand_chacha::ChaCha20Rng;
    type Solution = ScheduleSolution;

    fn generate_initial_solution(&self, rng: &mut Self::R) -> Self::Solution {
        let days = self.end_date.signed_duration_since(self.start_date).num_days() as u32 + 1;
        let mut date_to_employee = Vec::with_capacity(days as usize);
        for day in self.start_date.iter_days() {
            date_to_employee.push(*self.employees.choose(rng).unwrap());
            if day > self.end_date {
                break;
            }
        }
        Self::Solution {
            start_date: self.start_date,
            end_date: self.end_date,
            date_to_employee,
            employees: self.employees.clone(),
        }
    }
}

pub struct ScheduleMoveProposer {
    pub next_employees: HashMap<Employee, Employee>,
}

impl ScheduleMoveProposer {
    pub fn new(employees: BTreeSet<Employee>) -> Self {
        let mut next_employees = HashMap::with_capacity(employees.len());
        for employee in &employees {
            let next_employee = match &employees.range((Excluded(employee), Unbounded)).next() {
                None => *employees.iter().next().unwrap(),
                Some(found_next_employee) => **found_next_employee,
            };
            next_employees.insert(*employee, next_employee);
        }
        Self { next_employees }
    }
}

impl MoveProposer for ScheduleMoveProposer {
    type R = rand_chacha::ChaCha20Rng;
    type Solution = ScheduleSolution;

    fn iter_local_moves(
        &self,
        start: &Self::Solution,
        rng: &mut Self::R,
    ) -> Box<dyn Iterator<Item = Self::Solution>> {
        struct MoveIterator {
            current_day: usize,
            current_employee: Option<Employee>,
            solution: ScheduleSolution,
            next_employees: HashMap<Employee, Employee>,
        }
        impl Iterator for MoveIterator {
            type Item = ScheduleSolution;

            fn next(&mut self) -> Option<Self::Item> {
                if self.current_day >= self.solution.date_to_employee.len() {
                    return None;
                }
                let current_employee = match &self.current_employee {
                    None => &self.solution.date_to_employee[self.current_day],
                    Some(actual_current_employee) => actual_current_employee,
                };
                let next_employee = self.next_employees.get(current_employee).unwrap();
                let mut new_solution = self.solution.clone();
                new_solution.date_to_employee[self.current_day] = *next_employee;

                if self.solution.date_to_employee[self.current_day] == *next_employee {
                    self.current_day += 1;
                    self.current_employee = None;
                } else {
                    self.current_employee = Some(*next_employee);
                }

                Some(new_solution)
            }
        }

        Box::new(MoveIterator {
            current_day: 0,
            current_employee: None,
            solution: start.clone(),
            next_employees: self.next_employees.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchedulePerturbationStrategy {
    DoNothing,
    ChangeDaysSubsetRandomly,
}

pub struct SchedulePerturbation {
    strategy: Vec<(SchedulePerturbationStrategy, u64)>,
}

impl SchedulePerturbation {
    pub fn default() -> Self {
        Self {
            strategy: vec![
                (SchedulePerturbationStrategy::DoNothing, 10),
                (SchedulePerturbationStrategy::ChangeDaysSubsetRandomly, 100),
            ],
        }
    }
}

impl Perturbation for SchedulePerturbation {
    type _R = rand_chacha::ChaCha20Rng;
    type _Solution = ScheduleSolution;
    type _Score = ScheduleScore;
    type _SSC = ScheduleSolutionScoreCalculator;

    fn propose_new_starting_solution(
        &mut self,
        current: &ScoredSolution<Self::_Solution, Self::_Score>,
        history: &History<Self::_R, Self::_Solution, Self::_Score>,
        rng: &mut Self::_R,
    ) -> Self::_Solution {
        let current_strategy = self.strategy.choose_weighted(rng, |s| s.1).unwrap().0.clone();
        let mut new_solution = current.solution.clone();
        match current_strategy {
            SchedulePerturbationStrategy::DoNothing => new_solution,
            SchedulePerturbationStrategy::ChangeDaysSubsetRandomly => {
                let total_days = new_solution.date_to_employee.len();
                let number_of_days_to_alter = match history.is_best_solution(current.clone()) {
                    true => rng.gen_range(1..=(total_days / 20).clamp(1, total_days)),
                    false => rng.gen_range(1..=(total_days / 2).clamp(1, total_days)),
                };
                let mut indices: Vec<usize> = (0..total_days).collect();
                indices.shuffle(rng);
                for index in indices.into_iter().take(number_of_days_to_alter) {
                    new_solution.date_to_employee[index] = *new_solution.employees.choose(rng).unwrap();
                }
                new_solution
            }
        }
    }
}
