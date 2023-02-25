#[cfg(test)]
mod test {
    use crate::test::*;
    use crate::*;

    pub struct SchObjective1 {}

    impl Objective<CandidateF64> for SchObjective1 {
        fn value(&self, candidate: &CandidateF64) -> f64 {
            candidate.val * candidate.val
        }
    }

    pub struct SchObjective2 {}

    impl Objective<CandidateF64> for SchObjective2 {
        fn value(&self, candidate: &CandidateF64) -> f64 {
            let x = candidate.val - 2.;
            x * x
        }
    }

    #[test]
    fn sch() {
        let objectives: Vec<Box<dyn Objective<CandidateF64>>> =
            vec![Box::new(SchObjective1 {}), Box::new(SchObjective2 {})];

        let meta = ParamsF64 {
            population_size: POPULATION_SIZE,
            crossover_odds: &CROSSOVER_ODDS,
            mutation_odds: &Ratio(1, 1),
            objectives,
            constraints: vec![],
            val_range: -55.0..56.0,
        };

        let mut optimizer = NSGAOptimizer::new(meta);
        optimizer
            .optimize(Box::new(DefaultEvaluator::new(100)))
            .for_each(|x| {
                println!("{}", x.val);
                assert!(x.val >= -0.1 && x.val <= 2.1)
            });
    }
}
