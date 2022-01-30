# nsga

*nsga* is an opinionated implementation of the
[NSGA-II (Non-dominated Sorting Genetic Algorithm)](https://ieeexplore.ieee.abstract/document/996017),
a multi-objective genetic optimization algorithm.

The focus for this implementation is on practical applicability,
not necessarily just for optimizing pure mathematical functions.

# Example

Let's define an example problem:

```
Given an array of integers and a value, find indices of the array elements
that sum up to the given value.
```

For example, given an array:
```rust
let a = vec![1, 5, 8, 0, 6, 4];
```

and a value of `19`, the solution could be:
```rust
vec![1, 0, 1, 0, 1, 1]; // 1 + 8 + 6 + 4 = 19
```

or

```rust
vec![0, 1, 1, 0, 1, 0]; // 5 + 8 + 6 = 19
```

Of course, for such a small input we wouldn't need a fancy optimizer,
but when dealing with thousands or millions of elements, the task becomes
somewhat challenging.

## Solution candidate

The problem is represented by an implementation of the `Solution` trait.
Let's define a base structure for our solution candidate:

```rust
#[derive(Clone, Debug)]
struct Candidate {
  indices: Vec<isize>,
}
```

Since the solution to our problem is an array of indices, we can simply have
a `Vec<isize>` in our struct.

### Mutation

[Mutation](https://en.wikipedia.org/wiki/Mutation_(genetic_algorithm))
is an operation that changes the solution candidate.
It is a way for the system to add diversity and escape local optima.
Not unlike the role mutation plays in the evolution
of the real biological systems.

Mutation is heavily problem-dependent and in our case
we're just going to flip an index from 0 to 1 and vice versa with a
certain probability.

```rust
fn mutate(&mut self) {
  let mut rng = thread_rng();

  for i in &mut self.indices {
    if rng.gen_ratio(MUTATION_ODDS.0, MUTATION_ODDS.1) {
      *i = if *i == 0 { 1 } else { 0 }
    }
  }
}
```

### Crossover

[Crossover](https://en.wikipedia.org/wiki/Crossover_(genetic_algorithm))
is an operation that takes two parent candidates and
mixes their "genes". In our implementation, we're going to split both
parents in half and swap corresponding parts.
I.e. given two parents:

```rust
let a = vec![1, 1, 1, 1, 1, 1];
let b = vec![0, 0, 0, 0, 0, 0];
```

after the crossover these would look like:

```rust
vec![1, 1, 1, 0, 0, 0];
vec![0, 0, 0, 1, 1, 1];
```

```rust

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
```

## Objective

In order to guide the optimizer, we need to implement the `Objective` trait.
The only mandatory method is `Objective::value` which takes a solution
candidate and returns its fitness value. The lower this value the closer
a particular solution is to the ideal solution.

For our task we'd implement something like the following:

```rust
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
```

Basically, it computes the sum of all the values for which the
the index bit was set in the solution and then computes the difference
between the sum and the target value, we're looking for.

The closer the sum of the current solution is to the value
we're looking for, the smaller will be the difference, and this is exactly
what we need, since the optimizer always tries to find the function minimum.

### Early termination

When in a particular objective the target value is known,
the optimization process can be made significantly faster
by not having to compute all the iteration steps.

For example, in our case, we know exactly the value we're looking
for so we can terminate the search the moment we're close enough
to the desired value.

```rust
fn good_enough(&self, val: f64) -> bool {
   val <= self.toleration
}
```

By tweaking the `self.toleration` value we can make the search as
precise as we need.

## Metadata

There's a set of additional meta-parameters we'd need to provide to the
optimizer. We do this by implementing a `Meta` trait.

### Meta::population_size

Population size is the size of the internal pool of candidates optimizer uses.
The default value is `20` and in most cases, it can be left untouched.

### Meta::crossover_odds

This method should return a probability of applying a crossover operation.
It should generally be relatively high, around 50% or so.

### Meta::mutation_odds

This method should return a probability of applying a mutation operation.
It should generally be smaller than the crossover value, around 20-30% or so.

### Meta::random_solution

A method to return a random solution candidate.
In our case, we'll just return a vector of zeroes for the indices.

```rust
fn random_solution(&mut self) -> Candidate {
  let indices: Vec<isize> = (0..self.records_length).map(|_| 0).collect();

  Candidate { indices }
}
```

### Meta::objectives

This method returns a vector of objectives to use in the optimization.
In our case, it will be an instance of our `SumObjective` one:

```rust
fn objectives(&self) -> &Vec<Box<dyn Objective<Candidate>>> {
  vec![
    Box::new(SumObjective {
      goal: 19.,
      items: vec![1, 5, 8, 0, 6, 4],
      toleration: 0.0,
    }),
  ]
}
```

### Meta::constraints

This method returns an optional vector of constraints to use in the optimization.
We won't need constraints for our little example.

## Multiple objectives

Now, being able to optimize for one objective is great, but `NSGA-II` is a
multi-objective optimization algorithm, meaning that it can optimize
for many objectives at the same time. And some of those may even conflict
with each other! The details are outside the scope of this tutorial,
feel free to read more about it on
[Wikipedia](https://en.wikipedia.org/wiki/Multi-objective_optimization),
if you'd like.

Remember, with our initial test vector:
```rust
let a = vec![1, 5, 8, 0, 6, 4];
```

we identified two solutions with a sum of `19`:
```rust
let s1 = vec![1, 0, 1, 0, 1, 1]; // 1 + 8 + 6 + 4 = 19
let s2 = vec![0, 1, 1, 0, 1, 0]; // 5 + 8 + 6 = 19
```

Now, let's say in addition to finding a required sum, we'd also want
to find the one with the smallest number of summands.
So, for `s1` above there would be four summands: `1`, `8`, `6` and `4`,
while `s2` only has three: `5`, `8` and `6`, so we'd want our optimization
to find the latter one.

All we need for this is to implement another objective, let's call it
`OnesObjective`, because it's simply going to return the number of ones (set bits)
in the solution:

```rust
pub struct OnesObjective {}

impl Objective<Candidate> for OnesObjective {
    fn value(&self, candidate: &Candidate) -> f64 {
        candidate.indices.iter().filter(|i| **i == 1).count() as f64
    }
}
```

And then add to our `objectives` method:

```rust

fn objectives(&self) -> &Vec<Box<dyn Objective<Candidate>>> {
    vec![
        Box::new(SumObjective {
            goal: 19.,
            items: vec![1, 5, 8, 0, 6, 4],
            toleration: 0.0,
        }),
        Box::new(OnesObjective{}),
    ]
}
```

That's it!

For complete-code examples take a look at the crate tests:

* [test_sch](https://github.com/syhpoon/nsga/blob/master/src/test_sch.rs)
* [tes_sum](https://github.com/syhpoon/nsga/blob/master/src/test_sum.rs)

