use rand::rng;
use rand_distr::{Distribution, Poisson};

pub trait Generator {
    fn next_f64(&mut self) -> Option<f64>;
    fn next_bool(&mut self) -> Option<bool>;
}

/// Generates a sequence of boolean
#[derive(Debug, Clone)]
pub struct PoissonBinaryGenerator {
    d: Poisson<f64>,
}

impl PoissonBinaryGenerator {
    pub fn new(average_sequence_length: f64) -> Self {
        assert!(
            average_sequence_length != 0.0,
            "Average sequence length cannot be 0"
        );
        let lambda = 1.0 / average_sequence_length;
        Self {
            d: Poisson::new(lambda).unwrap(),
        }
    }
}

impl Generator for PoissonBinaryGenerator {
    fn next_bool(&mut self) -> Option<bool> {
        self.next_f64().map(|x| x <= 1.0)
    }

    fn next_f64(&mut self) -> Option<f64> {
        let roll = self.d.sample(&mut rng());
        Some(roll)
    }
}

#[cfg(test)]
mod tests {
    use crate::stat::generators::Generator;

    use super::PoissonBinaryGenerator;

    #[test]
    fn test_poisson_binary_gen() {
        let mut g = PoissonBinaryGenerator::new(3.0);
        let mut f = 0;
        let mut t = 0;
        for _ in 0..1000 {
            let b = g.next_bool().unwrap();
            if b {
                t = t + 1;
            } else {
                f = f + 1;
            }
            println!("Generated: {}", b);
        }
        assert!(t >= f)
    }
}
