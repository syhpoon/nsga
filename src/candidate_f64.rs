use crate::*;
use rand::{thread_rng, Rng};
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct CandidateF64 {
    pub val: f64,
    range: Range<f64>,
}

impl Solution for CandidateF64 {
    // SBX Crossover
    // https://youtu.be/7-NPqSvutr0?t=718
    // https://github.com/baopng/NSGA-II/blob/master/nsga2/utils.py#L89
    fn crossover(&mut self, other: &mut Self) {
        let crossover_param: f64 = 2.;
        let u: f64 = thread_rng().gen_range(0.0..1.0);

        let beta = if u <= 0.5 {
            (2. * u).powf(1. / (crossover_param + 1.))
        } else {
            (2. * (1. - u)).powf(-1. / (crossover_param + 1.))
        };

        let x1 = (self.val + other.val) / 2.;
        let x2 = ((self.val - other.val) / 2.).abs();

        self.val = x1 + beta * x2;
        other.val = x1 - beta * x2;
    }

    // Polynomial mutation
    // https://youtu.be/7-NPqSvutr0?t=916
    // https://github.com/baopng/NSGA-II/blob/master/nsga2/utils.py#L108
    fn mutate(&mut self) {
        let mutation_param: f64 = 5.;
        let u: f64 = thread_rng().gen_range(0.0..1.0);
        let r0: f64 = self.range.start;
        let r1: f64 = self.range.end;

        let delta = if u < 0.5 {
            (2. * u).powf(1. / (mutation_param + 1.)) - 1.
        } else {
            1. - (2. * (1. - u)).powf(-1. / (mutation_param + 1.))
        };

        if u < 0.5 {
            self.val += delta * (self.val - r0)
        } else {
            self.val += delta * (r1 - self.val)
        }

        if self.val < r0 {
            self.val = r0
        } else if self.val > r1 {
            self.val = r1
        }
    }
}

pub struct ParamsF64<'a> {
    pub population_size: usize,
    pub crossover_odds: &'a Ratio,
    pub mutation_odds: &'a Ratio,
    pub objectives: Vec<Box<dyn Objective<CandidateF64>>>,
    pub constraints: Vec<Box<dyn Constraint<CandidateF64>>>,
    pub val_range: Range<f64>,
}

impl<'a> Meta<'a, CandidateF64> for ParamsF64<'a> {
    fn population_size(&self) -> usize {
        self.population_size
    }

    fn crossover_odds(&self) -> &'a Ratio {
        self.crossover_odds
    }

    fn mutation_odds(&self) -> &'a Ratio {
        self.mutation_odds
    }

    fn random_solution(&mut self) -> CandidateF64 {
        CandidateF64 {
            val: thread_rng().gen_range(self.val_range.clone()),
            range: self.val_range.clone(),
        }
    }

    fn objectives(&self) -> &Vec<Box<dyn Objective<CandidateF64>>> {
        &self.objectives
    }

    fn constraints(&self) -> &Vec<Box<dyn Constraint<CandidateF64> + 'a>> {
        &self.constraints
    }
}
