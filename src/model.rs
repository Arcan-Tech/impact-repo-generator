#[derive(Debug)]
pub struct CCPair {
    pub f1: String,
    pub f2: String,
}

impl CCPair {
    pub fn new(f1: String, f2: String) -> Self {
        Self { f1, f2 }
    }

    fn random(m: u32) -> Self {
        let f1_index = rand::random::<u32>() % m;
        let f2_index = rand::random::<u32>() % m;
        let f1 = format!("f_{}", f1_index);
        let f2 = format!("f_{}", f2_index);
        Self::new(f1, f2)
    }
}

impl std::fmt::Display for CCPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.f1, self.f2)
    }
}
