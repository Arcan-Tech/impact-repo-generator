use itertools::Itertools;
use rand_distr::{Distribution, Normal};

#[derive()]
pub struct CCPair {
    pub f1: String,
    pub f2: String,
    pub p: f64,
    pub d: Normal<f64>,
}

impl CCPair {
    pub fn new(f1: String, f2: String, p: f64, d: Normal<f64>) -> Self {
        Self { f1, f2, p, d }
    }

    pub fn normal(f1: String, f2: String) -> Self {
        Self::new(f1, f2, 0.5, Normal::new(0.0, 1.0).unwrap())
    }

    fn random(m: u32) -> Self {
        let f1_index = rand::random::<u32>() % m;
        let f2_index = rand::random::<u32>() % m;
        let f1 = format!("f_{}", f1_index);
        let f2 = format!("f_{}", f2_index);
        Self::normal(f1, f2)
    }

    pub fn activate(&self) -> bool {
        self.d.sample(&mut rand::rng()) <= self.p
    }
}

impl std::fmt::Debug for CCPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CCPair {{ {}, {}, {} }}", self.f1, self.f2, self.p)
    }
}
impl std::fmt::Display for CCPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.f1, self.f2)
    }
}

pub struct CCModel {
    pairs: Vec<CCPair>,
}

impl CCModel {
    pub fn new(pairs: Vec<CCPair>) -> Self {
        Self { pairs }
    }
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    pub fn random(n: usize, m: u32) -> Self {
        let pairs = (0..n).map(|_| CCPair::random(m)).collect_vec();
        Self::new(pairs)
    }

    pub fn sample(&self) -> Vec<&CCPair> {
        self.pairs.iter().filter(|p| p.activate()).collect_vec()
    }
}

#[cfg(test)]
mod test {
    use super::{CCModel, CCPair};
    use itertools::Itertools;

    #[test]
    fn generate_random_pairs() {
        let pairs = (0..100).map(|_| CCPair::random(20)).collect_vec();
        println!("{:?}", pairs);
    }

    #[test]
    fn generate_random_model() {
        let m = CCModel::random(10, 20);
        for ele in m.sample() {
            println!("{}", ele)
        }
    }
}
