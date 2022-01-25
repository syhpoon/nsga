use peeking_take_while::PeekableExt;
use rand::prelude::*;
use rand::seq::SliceRandom;

use crate::{Evaluator, Objective};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::convert::identity;

use super::{Meta, Ratio, Solution, SolutionId};

#[derive(Debug, Clone)]
struct Candidate<S: Solution> {
    sol: S,
    front: usize,
    distance: f64,
}

pub struct NSGAOptimizer<'a, S: Solution> {
    meta: Box<dyn Meta<'a, S> + 'a>,
    last_id: SolutionId,
    best_solutions: Vec<(Vec<f64>, S)>,
}

impl<'a, S> NSGAOptimizer<'a, S>
where
    S: Solution,
{
    pub fn new(meta: impl Meta<'a, S> + 'a) -> Self {
        NSGAOptimizer {
            meta: Box::new(meta),
            last_id: 0,
            best_solutions: Vec::new(),
        }
    }

    pub fn optimize(&mut self, mut eval: Box<dyn Evaluator>) -> impl Iterator<Item = S> {
        let mut rnd = rand::thread_rng();

        let pop_size = self.meta.population_size();
        let crossover_odds = self.meta.crossover_odds();
        let mutation_odds = self.meta.mutation_odds();

        // Initial population
        let pop: Vec<_> = (0..pop_size)
            .map(|_| {
                let id = self.next_id();
                let sol = self.meta.random_solution(id);

                Candidate {
                    sol,
                    front: 0,
                    distance: 0.0,
                }
            })
            .collect();

        let mut parent_pop = self.sort(pop);

        for iter in 0.. {
            // Keep copies of the best candidates in a stash
            parent_pop
                .iter()
                .take_while(|c| c.front == 0)
                .for_each(|c| {
                    let vals: Vec<f64> = self.values(&c.sol);

                    // Only keep better old values
                    self.best_solutions
                        .retain(|s| s.0.iter().zip(&vals).any(|(old, new)| old < new));

                    self.best_solutions.push((vals, c.sol.clone()));
                });

            // Check if there's a good-enough solution already
            if parent_pop
                .iter()
                .map(|c| {
                    self.meta
                        .objectives()
                        .iter()
                        .map(|obj| obj.good_enough(self.value(&c.sol, obj)))
                        .all(identity)
                })
                .any(identity)
            {
                break;
            }

            // Check if we can already terminate
            if parent_pop
                .iter()
                .map(|c| eval.can_terminate(iter, self.values(&c.sol)))
                .any(|t| t)
            {
                break;
            }

            let mut child_pop: Vec<Candidate<S>> = Vec::with_capacity(pop_size);

            while child_pop.len() < pop_size {
                let p1 = parent_pop.choose_mut(&mut rnd).unwrap().clone();
                let p2 = parent_pop.choose_mut(&mut rnd).unwrap().clone();
                let p3 = parent_pop.choose_mut(&mut rnd).unwrap().clone();
                let p4 = parent_pop.choose_mut(&mut rnd).unwrap().clone();

                let mut c1 = self.tournament(p1, p2);
                let mut c2 = self.tournament(p3, p4);

                if self.odds(crossover_odds) {
                    c1.sol.crossover(&mut c2.sol);
                };

                if self.odds(mutation_odds) {
                    c1.sol.mutate();
                };

                if self.odds(mutation_odds) {
                    c2.sol.mutate();
                };

                c1.sol.set_id(self.next_id());
                c2.sol.set_id(self.next_id());

                child_pop.push(c1);
                child_pop.push(c2);
            }

            parent_pop.extend(child_pop);

            // Sort combined population
            let sorted = self.sort(parent_pop);
            let mut sorted_iter = sorted.into_iter().peekable();

            // Now select the next population
            let mut next_pop: Vec<_> = Vec::with_capacity(pop_size);
            let mut front = 0;

            while next_pop.len() != pop_size {
                let mut front_items: Vec<_> = sorted_iter
                    .by_ref()
                    .peeking_take_while(|i| i.front == front)
                    .collect();

                // Front fits entirely
                if next_pop.len() + front_items.len() < next_pop.capacity() {
                    next_pop.extend(front_items);

                    front += 1;
                } else {
                    front_items.sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());

                    let rest: Vec<_> = front_items.drain(..(pop_size - next_pop.len())).collect();

                    next_pop.extend(rest);
                }
            }

            parent_pop = next_pop;
        }

        let best = std::mem::take(&mut self.best_solutions);
        best.into_iter().map(|s| s.1)
    }

    fn next_id(&mut self) -> SolutionId {
        self.last_id += 1;
        self.last_id
    }

    fn odds(&self, ratio: &Ratio) -> bool {
        thread_rng().gen_ratio(ratio.0, ratio.1)
    }

    fn tournament(&self, p1: Candidate<S>, p2: Candidate<S>) -> Candidate<S> {
        let mut rnd = rand::thread_rng();

        if p1.front < p2.front {
            p1
        } else if p2.front < p1.front {
            p2
        } else if p1.distance > p2.distance {
            p1
        } else if p2.distance > p1.distance {
            p2
        } else {
            vec![p1, p2].remove(rnd.gen_range(0..=1))
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn sort(&self, pop: Vec<Candidate<S>>) -> Vec<Candidate<S>> {
        let mut dominates: HashMap<SolutionId, HashSet<SolutionId>> = HashMap::new();
        let mut dominated_by: HashMap<SolutionId, usize> = HashMap::new();

        let ids: Vec<_> = pop.iter().map(|c| c.sol.id()).collect();
        let mut sols: HashMap<SolutionId, S> =
            pop.into_iter().map(|c| (c.sol.id(), c.sol)).collect();

        let mut fronts: Vec<HashSet<SolutionId>> = vec![HashSet::new()];

        // Stage 1
        for i in 0..ids.len() {
            let i_id = ids[i];

            for j in i + 1..ids.len() {
                let j_id = ids[j];
                let sol_i = &sols[&i_id];
                let sol_j = &sols[&j_id];

                let r = if self.dominates(sol_i, sol_j) {
                    Some((i_id, j_id))
                } else if self.dominates(sol_j, sol_i) {
                    Some((j_id, i_id))
                } else {
                    None
                };

                if let Some((d, dby)) = r {
                    dominates.entry(d).or_insert_with(HashSet::new).insert(dby);
                    *dominated_by.entry(dby).or_insert(0) += 1;
                }
            }

            if dominated_by.get(&i_id).is_none() {
                fronts[0].insert(i_id);
            }
        }

        // Stage2
        let mut i = 0;
        while !fronts[i].is_empty() {
            let mut new_front = HashSet::new();

            for id in fronts[i].iter() {
                if let Some(set) = dominates.get(id) {
                    for dominated_id in set.iter() {
                        dominated_by.entry(*dominated_id).and_modify(|v| {
                            if v > &mut 0 {
                                *v -= 1
                            }
                        });

                        match dominated_by.get(dominated_id) {
                            None | Some(0) => {
                                if !new_front.contains(dominated_id) {
                                    new_front.insert(*dominated_id);
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }

            i += 1;
            fronts.push(new_front);
        }

        let mut flat_fronts: Vec<Candidate<S>> = Vec::with_capacity(fronts.len());
        for (fidx, f) in fronts.into_iter().enumerate() {
            for id in f {
                let sol = sols.remove(&id).unwrap();

                flat_fronts.push(Candidate {
                    sol,
                    front: fidx,
                    distance: 0.0,
                });
            }
        }

        let mut fronts = flat_fronts;
        debug_assert!(!fronts.is_empty());

        // Crowding distance
        let fronts_len = fronts.len();

        for obj in self.meta.objectives() {
            // Sort by objective
            fronts.sort_by(|a, b| {
                let a_obj = self.value(&a.sol, obj);
                let b_obj = self.value(&b.sol, obj);

                a_obj.partial_cmp(&b_obj).unwrap()
            });

            let min = self.value(&fronts[0].sol, obj);
            let max = self.value(&fronts[fronts_len - 1].sol, obj);

            let mut diff = (max - min) as f64;
            if diff == 0. {
                diff = 1.
            }

            fronts[0].distance = f64::MAX;
            fronts[fronts_len - 1].distance = f64::MAX;

            for i in 2..fronts_len - 2 {
                if fronts[i].distance != f64::MAX {
                    fronts[i].distance += (obj.value(&fronts[i + 1].sol)
                        - obj.value(&fronts[i - 1].sol))
                    .abs() as f64
                        / diff;
                }
            }
        }

        // First sort by front and then by distance
        fronts.sort_by(|a, b| {
            if a.front != b.front {
                a.front.cmp(&b.front)
            } else if a.distance != b.distance {
                a.distance.partial_cmp(&b.distance).unwrap()
            } else {
                Ordering::Equal
            }
        });

        fronts
    }

    #[allow(clippy::borrowed_box)]
    fn value(&self, s: &S, obj: &Box<dyn Objective<S> + 'a>) -> f64 {
        self.meta
            .constraints()
            .iter()
            .fold(obj.value(s), |acc, cons| cons.value(s, acc))
    }

    fn values(&self, s: &S) -> Vec<f64> {
        self.meta
            .objectives()
            .iter()
            .map(|obj| self.value(s, obj))
            .collect()
    }

    fn dominates(&self, s1: &S, s2: &S) -> bool {
        let vals1 = self.values(s1);
        let vals2 = self.values(s2);

        let vals: Vec<_> = vals1.into_iter().zip(vals2).collect();

        vals.iter().all(|(v1, v2)| v1 <= v2) && vals.iter().any(|(v1, v2)| v1 < v2)
    }
}
