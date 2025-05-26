use std::path::Path;

use git::GitWriter;
use input::Probabilities;

pub mod args;
pub mod git;
pub mod input;
pub mod model;

fn main() {
    let m = Probabilities::from_yaml("./test_data/probs.yaml").unwrap();
    let mut gw = GitWriter::new(Path::new("/tmp/synth-repo")).unwrap();
    for i in 0..10 {
        let pairs = m.roll_cochanges();
        gw.modify_and_commit(&pairs).unwrap()
    }
}
