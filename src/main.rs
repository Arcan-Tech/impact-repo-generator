use std::path::Path;

use git::GitWriter;
use model::CCModel;

pub mod git;
pub mod model;

fn main() {
    let m = CCModel::random(10, 10);
    let mut gw = GitWriter::new(Path::new("/tmp/synth-repo")).unwrap();
    for i in 0..10 {
        let pairs = m.sample();
        gw.modify_and_commit(&pairs).unwrap()
    }
}
