mod candidate_f64_pair;

use candidate_f64_pair::{CandidateF64Pair, ParamsF64Pair};
use nsga::*;
use std::env;

pub struct Objective1 {}
impl Objective<CandidateF64Pair> for Objective1 {
    fn value(&self, candidate: &CandidateF64Pair) -> f64 {
        4. * &candidate.x.val.powi(2) + 4. * &candidate.y.val.powi(2)
    }
}

pub struct Objective2 {}
impl Objective<CandidateF64Pair> for Objective2 {
    fn value(&self, candidate: &CandidateF64Pair) -> f64 {
        (&candidate.x.val - 5.).powi(2) + (&candidate.y.val - 5.).powi(2)
    }
}

pub struct Constraint1 {}
impl Constraint<CandidateF64Pair> for Constraint1 {
    fn value(&self, candidate: &CandidateF64Pair, val: f64) -> f64 {
        let cval = (candidate.x.val - 5.).powi(2) + candidate.y.val.powi(2);
        if cval > 25. {
            f64::MAX
        } else {
            val
        }
    }
}

pub struct Constraint2 {}
impl Constraint<CandidateF64Pair> for Constraint2 {
    fn value(&self, candidate: &CandidateF64Pair, val: f64) -> f64 {
        let cval = (candidate.x.val - 8.).powi(2) + (candidate.y.val + 3.).powi(2);

        if cval < 7.7 {
            f64::MAX
        } else {
            val
        }
    }
}

fn main() {
    const POPULATION_SIZE: usize = 20;
    const CROSSOVER_ODDS: Ratio = Ratio(6, 10);
    const MUTATION_ODDS: Ratio = Ratio(1, 1);

    let objectives: Vec<Box<dyn Objective<CandidateF64Pair>>> =
        vec![Box::new(Objective1 {}), Box::new(Objective2 {})];

    let constraints: Vec<Box<dyn Constraint<CandidateF64Pair>>> =
        vec![Box::new(Constraint1 {}), Box::new(Constraint2 {})];

    let meta = ParamsF64Pair {
        population_size: POPULATION_SIZE,
        crossover_odds: &CROSSOVER_ODDS,
        mutation_odds: &MUTATION_ODDS,
        objectives,
        constraints,
        val_range_x: 0.0..=5.0,
        val_range_y: 0.0..=3.0,
    };

    let mut samples = 10;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        samples = args[1]
            .parse()
            .expect("failed to parse the number of samples");
    }

    println!(
        "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
        "x", "y", "f1(x, y)", "f2(x, y)"
    );
    println!("===============================================");

    let mut optimizer = NSGAOptimizer::new(meta);
    optimizer
        .optimize(Box::new(DefaultEvaluator::new(100)))
        .take(samples)
        .for_each(|s| {
            let x = (4. * s.x.val.powi(2)) + (4. * s.y.val.powi(2));
            let y = (s.x.val - 5.).powi(2) + (s.y.val - 5.).powi(2);

            println!(
                "{0: <10.4} | {1: <10.4} | {2: <10.4} | {3: <10.4}",
                s.x.val, s.y.val, x, y
            );
        });
}
