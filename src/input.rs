use core::f64;
use itertools::Itertools;
use rand_distr::Distribution;
use serde::Deserialize;
use statrs::distribution::{ContinuousCDF, DiscreteCDF};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct CCPair {
    pub f1: String,
    pub f2: String,
}

impl CCPair {
    pub fn new(f1: String, f2: String) -> Self {
        Self { f1, f2 }
    }
}

impl std::fmt::Display for CCPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.f1, self.f2)
    }
}

#[derive(Debug, Deserialize)]
pub enum Distr {
    Poisson { lambda: f64 },
    Normal { mean: f64, variance: f64 },
}

impl Distr {
    pub fn sample(&self) -> f64 {
        match self {
            &Distr::Poisson { lambda } => {
                let d_sample = rand_distr::Poisson::new(lambda).unwrap();
                d_sample.sample(&mut rand::rng())
            }
            &Distr::Normal { mean, variance } => {
                let d_sample = rand_distr::Normal::new(mean, variance).unwrap();
                d_sample.sample(&mut rand::rng())
            }
        }
    }

    pub fn cdf(&self, x: f64) -> f64 {
        match self {
            &Distr::Poisson { lambda } => {
                let d_cdf = statrs::distribution::Poisson::new(lambda).unwrap();
                d_cdf.cdf(x.round() as u64)
            }
            &Distr::Normal { mean, variance } => {
                let d_cdf = statrs::distribution::Normal::new(mean, variance).unwrap();
                d_cdf.cdf(x)
            }
        }
    }

    pub fn p_less(&self, p: f64) -> bool {
        let x = self.sample();
        self.cdf(x) <= p
    }
}

#[derive(Debug, Deserialize)]
pub struct CCModel {
    pub probabilites: Vec<RippleProbability>,
    pub distribution: Distr,
}

impl CCModel {
    pub fn from_yaml<P>(path: P) -> Result<CCModel, Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        let contents = fs::read_to_string(path)?;
        let data: CCModel = serde_yaml::from_str(&contents)?;
        for p in &data.probabilites {
            p.f2.iter().for_each(|(f, &v)| {
                assert!(
                    v >= 0.0 && v <= 1.0,
                    "Error: Value {v} must be within 0 and 1 (included) for f1={} and f2={}",
                    p.f1,
                    f
                )
            });
        }
        Ok(data)
    }

    pub fn new(probs: Vec<RippleProbability>, distr: Distr) -> Self {
        Self {
            probabilites: probs,
            distribution: distr,
        }
    }

    pub fn roll_cochanges(&self) -> Vec<CCPair> {
        self.probabilites
            .iter()
            .flat_map(|r| {
                r.f2.iter().filter_map(|(k, &p)| {
                    if self.distribution.p_less(p) {
                        Some(CCPair::new(r.f1.clone(), k.clone()))
                    } else {
                        None
                    }
                })
            })
            .collect_vec()
    }
}

#[derive(Debug, Deserialize)]
pub struct RippleProbability {
    pub f1: String,
    pub f2: HashMap<String, f64>,
}

#[cfg(test)]
mod tests {
    use super::{CCModel, Distr};

    #[test]
    fn test_sample() {
        let d = Distr::Poisson { lambda: 3.0 };
        println!("Poisson");
        for _i in 1..10 {
            let x = d.sample();
            let cdf = d.cdf(x);
            println!("x = {}, cdf = {:.2}, cdf < .1 = {}", x, cdf, cdf <= 0.1)
        }
        let d = Distr::Normal {
            mean: 0.0,
            variance: 1.0,
        };
        println!("Normal");
        for _i in 1..10 {
            let x = d.sample();
            let cdf = d.cdf(x);
            println!("x = {}, cdf = {:.2}, cdf < .1 = {}", x, cdf, cdf <= 0.1)
        }
    }

    #[test]
    fn test_read() {
        let pfile = CCModel::from_yaml("./test_data/probs.yaml").unwrap();
        assert_eq!(3, pfile.probabilites.len());
        for pair in pfile.roll_cochanges() {
            println!("{}", pair);
        }
    }
}
