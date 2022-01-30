/// Evaluate the termination condition
pub trait Evaluator {
    /// Returns true if an optimization process can be stopped
    fn can_terminate(&mut self, iter: usize, values: Vec<f64>) -> bool;
}

/// Implements a default termination condition.
///
/// It saves best solutions on every iteration
/// and returns when there was no improvement for the provided
/// `terminate_early` iterations.
pub struct DefaultEvaluator {
    terminate_early: usize,
    iter: usize,
    values: Option<Vec<f64>>,
    no_improvements: usize,
}

impl DefaultEvaluator {
    pub fn new(terminate_early: usize) -> Self {
        DefaultEvaluator {
            terminate_early,
            iter: 0,
            values: None,
            no_improvements: 0,
        }
    }
}

impl Evaluator for DefaultEvaluator {
    fn can_terminate(&mut self, iter: usize, values: Vec<f64>) -> bool {
        match &self.values {
            None => {
                self.values = Some(values);
                self.iter = iter;

                false
            }
            Some(old_best) => {
                let has_better = old_best
                    .iter()
                    .zip(values.iter())
                    .any(|(old, new)| new < old);

                let no_worse = old_best
                    .iter()
                    .zip(values.iter())
                    .all(|(old, new)| new <= old);

                if has_better && no_worse {
                    self.values = Some(values);
                    self.iter = iter;
                    self.no_improvements = 0;

                    false
                } else if iter == self.iter {
                    false
                } else {
                    self.iter = iter;
                    self.no_improvements += 1;
                    self.no_improvements >= self.terminate_early
                }
            }
        }
    }
}
