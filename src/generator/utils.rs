use chrono::{Duration, NaiveDateTime};
use rand::rng;
use rand_distr::{Distribution, Exp};

pub trait Generator {
    fn next_f64(&mut self) -> Option<f64>;
    fn next_bool(&mut self) -> Option<bool>;
}

pub struct TimestampGenerator {
    start: i64,
    exp_gen: ExpGenerator,
}

impl TimestampGenerator {
    pub fn new(start: NaiveDateTime, average_interval: Duration) -> Self {
        let exp_gen = ExpGenerator::new(average_interval.as_seconds_f64());
        Self {
            start: start.and_utc().timestamp(),
            exp_gen,
        }
    }
}

impl Iterator for TimestampGenerator {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        let interval = self.exp_gen.next().unwrap_or(0.0);
        self.start = self.start + interval as i64;
        Some(self.start)
    }
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

impl Iterator for ExpGenerator {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let roll = self.d.sample(&mut rng());
        Some(roll)
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::{markov::StateExpSequence, state::State};

    use super::ExpGenerator;
    use rand_distr::num_traits::Inv;
    use std::collections::HashMap;

    #[test]
    fn test_poisson_binary_gen() {
        let avrg_seq_len = 10.0;
        let mut g = ExpGenerator::new(avrg_seq_len);
        let mut f = 0;
        let mut t = 0;
        let n = 1000;
        for _ in 0..n {
            let b = g.next().map(|x| x <= 1.0).unwrap_or(false);
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
