use std::collections::HashMap;

use rand::rng;
use rand_distr::{Distribution, Exp};
use serde::Deserialize;

use super::markov::State;

pub trait Generator {
    fn next_f64(&mut self) -> Option<f64>;
    fn next_bool(&mut self) -> Option<bool>;
}

/// Samples values from an exponential distribution
#[derive(Debug, Clone)]
pub struct ExpGenerator {
    d: Exp<f64>,
}

impl ExpGenerator {
    pub fn new(average_sequence_length: f64) -> Self {
        assert!(
            average_sequence_length != 0.0,
            "Average sequence length cannot be 0"
        );
        let lambda = 1.0 / average_sequence_length;
        Self {
            d: Exp::new(lambda).unwrap(),
        }
    }
}

impl Generator for ExpGenerator {
    fn next_bool(&mut self) -> Option<bool> {
        self.next_f64().map(|x| x <= 1.0)
    }

    fn next_f64(&mut self) -> Option<f64> {
        let roll = self.d.sample(&mut rng());
        Some(roll)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct StateExpSequence {
    #[serde(alias = "average_consecutive_commits")]
    sequence_length: HashMap<State, f64>,
    #[serde(skip, default = "Option::default")]
    current_state: Option<State>,
    #[serde(skip, default = "default_exp_gen")]
    seq_gen: ExpGenerator,
    #[serde(default = "default_seq_len")]
    default_seq_len: f64,
}

pub fn default_exp_gen() -> ExpGenerator {
    ExpGenerator::new(default_seq_len())
}

pub fn default_seq_len() -> f64 {
    1.0
}

impl StateExpSequence {
    pub fn new(avrg_seq_len: HashMap<State, f64>) -> Self {
        Self {
            sequence_length: avrg_seq_len,
            current_state: None,
            seq_gen: ExpGenerator::new(default_seq_len()),
            default_seq_len: default_seq_len(),
        }
    }

    /// Accepts the possibly new next state and returns either
    /// the previous state or the new state based on an exponentional probability
    /// distribution set on the current state's average sequence length.
    pub fn next_in_sequence(&mut self, next: &State) -> State {
        let swap_state = self.seq_gen.next_bool().unwrap();
        if self.current_state.is_none() || swap_state {
            dbg!(&next);
            self.current_state.replace(next.clone());
            let average_sequence_length = *self
                .sequence_length
                .get(next)
                .unwrap_or(&self.default_seq_len);
            self.seq_gen = ExpGenerator::new(average_sequence_length);
        }
        return self.current_state.clone().unwrap();
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rand_distr::num_traits::Inv;

    use crate::stat::{generators::Generator, markov::State};

    use super::{ExpGenerator, StateExpSequence};

    #[test]
    fn test_poisson_binary_gen() {
        let avrg_seq_len = 10.0;
        let mut g = ExpGenerator::new(avrg_seq_len);
        let mut f = 0;
        let mut t = 0;
        let n = 1000;
        for _ in 0..n {
            let b = g.next_bool().unwrap();
            if b {
                t = t + 1;
            } else {
                f = f + 1;
            }
            println!("Generated: {}", b);
        }
        assert!(t > 0);
        assert!(f > 0);
        assert!(f >= t);
        let x = t as f64 / n as f64;
        let delta = (x - avrg_seq_len.inv()).abs();
        assert!(delta <= 0.1, "Delta is more than 1: {}", delta);
    }

    #[test]
    fn test_poisson_sequence() {
        let issue1: State = State::Issue("issue1".to_string());
        let issue2: State = State::Issue("issue2".to_string());
        let m: HashMap<State, f64> = vec![(issue1.clone(), 5.0), (issue2.clone(), 10.0)]
            .into_iter()
            .collect();
        let mut seq = StateExpSequence::new(m);
        let mut i1 = 0;
        let mut i2 = 0;
        let n = 10000;
        for i in 0..n {
            let next = if i % 2 == 0 {
                issue1.clone()
            } else {
                issue2.clone()
            };
            let x = seq.next_in_sequence(&next);
            if x == issue1 {
                i1 = i1 + 1;
            } else if x == issue2 {
                i2 = i2 + 1;
            } else {
                assert!(false, "Unreachable");
            }
        }
        let i1 = i1 as f64 / (n as f64);
        let i2 = i2 as f64 / (n as f64);
        assert!(i1 > 0.0);
        assert!(i2 > 0.0);
        let delta = (i2 - (i1 * 2.0)).abs();
        assert!(delta < 0.2, "Delta is not smaller than 0.1: {}", delta)
    }
}
