use nsga::{CandidateF64, Constraint, Meta, Objective, Ratio, Solution};
use rand::{thread_rng, Rng};
use std::ops::RangeInclusive;

#[derive(Clone, Debug)]
pub struct CandidateF64Pair {
    pub x: CandidateF64,
    pub y: CandidateF64,
}

impl Solution for CandidateF64Pair {
    fn crossover(&mut self, other: &mut Self) {
        self.x.crossover(&mut other.x);
        self.y.crossover(&mut other.y);
    }

    fn mutate(&mut self) {
        self.x.mutate();
        self.y.mutate();
    }
}

pub struct ParamsF64Pair<'a> {
    pub population_size: usize,
    pub crossover_odds: &'a Ratio,
    pub mutation_odds: &'a Ratio,
    pub objectives: Vec<Box<dyn Objective<CandidateF64Pair>>>,
    pub constraints: Vec<Box<dyn Constraint<CandidateF64Pair>>>,
    pub val_range_x: RangeInclusive<f64>,
    pub val_range_y: RangeInclusive<f64>,
}

impl<'a> Meta<'a, CandidateF64Pair> for ParamsF64Pair<'a> {
    fn population_size(&self) -> usize {
        self.population_size
    }

    fn crossover_odds(&self) -> &'a Ratio {
        self.crossover_odds
    }

    fn mutation_odds(&self) -> &'a Ratio {
        self.mutation_odds
    }

    fn random_solution(&mut self) -> CandidateF64Pair {
        let x = CandidateF64 {
            val: thread_rng().gen_range(self.val_range_x.clone()),
            range_start: *self.val_range_x.start(),
            range_end: *self.val_range_x.end(),
        };

        let y = CandidateF64 {
            val: thread_rng().gen_range(self.val_range_y.clone()),
            range_start: *self.val_range_x.start(),
            range_end: *self.val_range_x.end(),
        };

        CandidateF64Pair { x, y }
    }

    fn objectives(&self) -> &Vec<Box<dyn Objective<CandidateF64Pair>>> {
        &self.objectives
    }

    fn constraints(&self) -> &Vec<Box<dyn Constraint<CandidateF64Pair> + 'a>> {
        &self.constraints
    }
}
