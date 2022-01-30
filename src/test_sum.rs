#[cfg(test)]
mod test {
    use crate::test::*;
    use crate::*;
    use rand::{thread_rng, Rng};

    #[derive(Clone, Debug)]
    struct Candidate {
        indices: Vec<isize>,
    }

    impl Solution for Candidate {
        fn crossover(&mut self, other: &mut Self) {
            let mut a = &mut self.indices;
            let mut b = &mut other.indices;

            // Use `a` for the longer vector:
            if b.len() > a.len() {
                a = &mut other.indices;
                b = &mut self.indices;
            }

            let a_mid = a.len() / 2;
            let b_mid = b.len() / 2;

            let b_back = &mut b[b_mid..];
            let a_back = &mut a[a_mid..][..b_back.len()];

            a_back.swap_with_slice(b_back);

            let a_len = a_mid + a_back.len();
            b.extend(a.drain(a_len..));
        }

        fn mutate(&mut self) {
            let mut rng = thread_rng();

            for i in &mut self.indices {
                if rng.gen_ratio(MUTATION_ODDS.0, MUTATION_ODDS.1) {
                    *i = if *i == 0 { 1 } else { 0 }
                }
            }
        }
    }

    pub struct SumObjective {
        goal: f64,
        items: Vec<f64>,
        toleration: f64,
    }

    impl Objective<Candidate> for SumObjective {
        fn value(&self, candidate: &Candidate) -> f64 {
            let res: f64 = candidate
                .indices
                .iter()
                .enumerate()
                .map(|(i, rec)| if *rec == 1 { self.items[i] } else { 0. })
                .sum();

            let diff = (self.goal - res).abs();
            if diff < 0. {
                f64::MAX
            } else {
                diff
            }
        }

        fn good_enough(&self, val: f64) -> bool {
            val <= self.toleration
        }
    }

    pub struct OnesObjective {}

    impl Objective<Candidate> for OnesObjective {
        fn value(&self, candidate: &Candidate) -> f64 {
            candidate.indices.iter().filter(|i| **i == 1).count() as f64
        }
    }

    struct Params {
        records_length: usize,
        objectives: Vec<Box<dyn Objective<Candidate>>>,
        constraints: Vec<Box<dyn Constraint<Candidate>>>,
    }

    impl<'a> Meta<'a, Candidate> for Params {
        fn population_size(&self) -> usize {
            POPULATION_SIZE
        }

        fn crossover_odds(&self) -> &'a Ratio {
            &CROSSOVER_ODDS
        }

        fn mutation_odds(&self) -> &'a Ratio {
            &Ratio(1, 1)
        }

        fn random_solution(&mut self) -> Candidate {
            let indices: Vec<isize> = (0..self.records_length).map(|_| 0).collect();

            Candidate { indices }
        }

        fn objectives(&self) -> &Vec<Box<dyn Objective<Candidate>>> {
            &self.objectives
        }

        fn constraints(&self) -> &Vec<Box<dyn Constraint<Candidate>>> {
            &self.constraints
        }
    }

    struct TestCase {
        answer: Vec<isize>,
        objectives: Vec<Box<dyn Objective<Candidate>>>,
        constraints: Vec<Box<dyn Constraint<Candidate>>>,
    }

    #[test]
    fn sum() {
        let cases = vec![
            TestCase {
                answer: vec![1, 0, 1, 1, 0, 1, 0, 1, 1],
                objectives: vec![Box::new(SumObjective {
                    goal: 100.,
                    items: vec![90., 15., 1., 2., 20., 5., 30., 1., 1.],
                    toleration: 0.0,
                })],
                constraints: vec![],
            },
            TestCase {
                answer: vec![0, 0, 0, 0, 1, 0, 0, 0, 0, 1],
                objectives: vec![
                    Box::new(SumObjective {
                        goal: 100.,
                        items: vec![10., 20., 20., 10., 95., 10., 10., 10., 10., 5.],
                        toleration: 0.0,
                    }),
                    Box::new(OnesObjective {}),
                ],
                constraints: vec![],
            },
            TestCase {
                answer: vec![0, 1, 1, 0, 1, 0],
                objectives: vec![
                    Box::new(SumObjective {
                        goal: 19.,
                        items: vec![1., 5., 8., 0., 6., 4.],
                        toleration: 0.0,
                    }),
                    Box::new(OnesObjective {}),
                ],
                constraints: vec![],
            },
        ];

        for c in cases.into_iter() {
            let meta = Params {
                records_length: c.answer.len(),
                objectives: c.objectives,
                constraints: c.constraints,
            };

            let mut optimizer = NSGAOptimizer::new(meta);
            let res = optimizer
                .optimize(Box::new(DefaultEvaluator::new(500)))
                .next()
                .unwrap();

            assert_eq!(c.answer, res.indices);
        }
    }
}
