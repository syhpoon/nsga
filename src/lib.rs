mod candidate_f64;
mod evaluator;
mod optimizer;

use std::fmt::Debug;

pub use candidate_f64::{CandidateF64, ParamsF64};
pub use evaluator::{DefaultEvaluator, Evaluator};
pub use optimizer::NSGAOptimizer;

pub struct Ratio(pub u32, pub u32);

pub type SolutionId = u64;

pub trait Solution: Debug + Clone {
    fn set_id(&mut self, id: SolutionId);
    fn id(&self) -> SolutionId;
    fn crossover(&mut self, other: &mut Self);
    fn mutate(&mut self);
}

pub trait Objective<S: Solution> {
    fn value(&self, sol: &S) -> f64;
    fn good_enough(&self, _val: f64) -> bool {
        false
    }
}

pub trait Constraint<S: Solution> {
    fn value(&self, sol: &S, val: f64) -> f64;
}

pub trait Meta<'a, S: Solution> {
    fn population_size(&self) -> usize;
    fn crossover_odds(&self) -> &'a Ratio;
    fn mutation_odds(&self) -> &'a Ratio;
    fn random_solution(&mut self, id: SolutionId) -> S;
    fn objectives(&self) -> &Vec<Box<dyn Objective<S> + 'a>>;
    fn constraints(&self) -> &Vec<Box<dyn Constraint<S> + 'a>>;
}

#[cfg(test)]
mod test {
    use crate::Ratio;

    pub(crate) const POPULATION_SIZE: usize = 20;
    pub(crate) const CROSSOVER_ODDS: Ratio = Ratio(1, 2);
    pub(crate) const MUTATION_ODDS: Ratio = Ratio(1, 20);
}

mod test_sch;
mod test_sum;
