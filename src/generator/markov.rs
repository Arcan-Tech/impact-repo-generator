use std::{collections::HashMap, fmt::Display, fs, path::Path, u32};

use rand::rng;
use rand_distr::{Distribution, Uniform};
use serde::Deserialize;

use super::{
    state::{State, TMatrix},
    utils::ExpGenerator,
};

#[derive(Debug, Clone, Deserialize)]
pub struct MarkovProcess {
    #[serde(skip)]
    current: State,
    #[serde(skip)]
    starting: State,
    pub transitions: TMatrix,
    issue_sequence: StateExpSequence,
    #[serde(skip, default = "default_uniform")]
    d: Uniform<f32>,
}

fn default_uniform() -> Uniform<f32> {
    Uniform::new(0.0, 1.0).unwrap()
}

impl MarkovProcess {
    pub fn new(m: TMatrix, starting: State) -> anyhow::Result<Self> {
        Ok(Self {
            current: starting.clone(),
            starting,
            issue_sequence: StateExpSequence::new(HashMap::new()), // TODO: model this from the rust
            // api
            transitions: m,
            d: default_uniform(),
        })
    }

    pub fn from_yaml<P: AsRef<Path>>(p: P) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(p)?;
        let mut mp: Self = serde_yaml::from_str(&contents)?;
        let mut to_add = Vec::new();
        for ts in mp.transitions.iter_transitions_mut() {
            ts.sort();
        }
        for ts in mp.transitions.iter_transitions() {
            for s in ts {
                if !mp.transitions.contains_state(&s.to) {
                    to_add.push(s.to.clone());
                }
            }
        }
        for s in to_add.into_iter() {
            mp.transitions.add_state(s);
        }
        mp.transitions.validate()?;
        Ok(mp)
    }

    /// Transitions to the next state. If the current state is an
    /// `State::Initial` and the next selected state is `State::Issue`
    /// then the next issue is selected using an exponential distribution.
    pub fn transition_next(&mut self) -> Option<&State> {
        let x = self.d.sample(&mut rng());
        if let Some(next) = self.transitions.next(&self.current, x) {
            let next = if self.current.is_initial() && next.is_issue() {
                self.issue_sequence.next_in_sequence(&next)
            } else {
                next
            };
            self.current = next;
            return Some(&self.current);
        }
        return None;
    }

    pub fn path_steps(&mut self, steps: usize) -> MarkovPath {
        let mut path = MarkovPath::new();
        path.append(self.current.clone());
        for _ in 0..steps {
            if let Some(s) = self.transition_next() {
                path.append(s.clone());
            }
        }
        path
    }

    pub fn path_until(&mut self, state: &State) -> MarkovPath {
        let mut path = MarkovPath::new();
        path.append(self.current.clone());
        if *state == self.current {
            return path;
        }
        loop {
            if let Some(s) = self.transition_next() {
                path.append(s.clone());
                if *state == *s {
                    break;
                }
            } else {
                break;
            }
        }
        path
    }

    pub fn reset(&mut self) {
        self.current = self.starting.clone();
    }

    pub fn num_states(&self) -> usize {
        self.transitions.num_states()
    }

    pub fn num_transitions(&self) -> usize {
        self.transitions.num_transitions()
    }

    pub fn as_edge_tuples(&self) -> Vec<(&State, f32, &State)> {
        self.transitions.as_edge_tuples()
    }

    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph MarkovProcess {\n    rankdir=LR;\n    node [shape=circle];\n");
        for (from, prob, to) in self.as_edge_tuples() {
            dot.push_str(&format!(
                "    \"{}\" -> \"{}\" [label=\"{:.2}\"];\n",
                from.name(),
                to.name(),
                prob
            ));
        }
        dot.push_str("}");
        dot
    }


}

impl Display for MarkovProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (from, p, to) in self.as_edge_tuples().iter() {
            writeln!(f, "{} -[{:.2}]-> {}", from, p, to)?;
        }
        Ok(())
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
        let swap_state = self.seq_gen.next().map(|x| x <= 1.0).unwrap_or(false);
        if self.current_state.is_none() || swap_state {
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

#[derive(Debug, Clone)]
pub struct MarkovPath {
    path: Vec<State>,
    seen: HashMap<State, u32>,
}

impl MarkovPath {
    pub fn new() -> Self {
        MarkovPath {
            path: Vec::new(),
            seen: HashMap::new(),
        }
    }

    pub fn append(&mut self, s: State) -> &mut Self {
        if !self.seen.contains_key(&s) {
            self.path.push(s.clone());
            self.seen.insert(s, 1);
        } else {
            self.seen.get_mut(&s).map(|x| *x = *x + 1);
        }
        self
    }
    pub fn iter(&self) -> impl Iterator<Item = &State> {
        self.path.iter()
    }

    pub fn iter_count(&self) -> impl Iterator<Item = (&State, u32)> {
        self.path.iter().map(|s| (s, *self.seen.get(s).unwrap()))
    }

    pub fn len(&self) -> usize {
        self.path.len()
    }
}

impl Display for MarkovPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.is_empty() {
            writeln!(f, "Path[empty]")?;
            return Ok(());
        }
        write!(f, "Path[")?;
        for (e, x) in self.iter_count().take(self.len() - 1) {
            write!(f, "({}, {}) -> ", e, x)?
        }
        let (e, x) = self.iter_count().last().unwrap();
        writeln!(f, "({}, {})]", e, x)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::{
        markov::{MarkovProcess, State},
        state::TMatrix,
    };

    #[test]
    fn test_process() {
        let mut m = TMatrix::new();
        let states: Vec<State> = vec!["file1".into(), "file2".into()];
        m.add_states(states.clone());
        m.set_transitions("file1", vec![("file1", 0.5), ("file2", 0.5)]);
        m.set_transitions("file2", vec![("file1", 0.80), ("file2", 0.2)]);
        let mut mp = MarkovProcess::new(m, states[0].clone()).unwrap();
        for _i in 0..100 {
            let sn = mp.transition_next();
            assert!(sn.is_some());
            let name = sn.unwrap();
            let name = name.name();
            assert!(name == "file1" || name == "file2");
        }

        let mut m = TMatrix::new();
        let states: Vec<State> = vec!["file1".into(), "file2".into(), "file3".into()];
        m.add_states(states.clone());
        m.set_transitions("file1", vec![("file2", 0.5), ("file3", 0.5)]);
        let mut mp = MarkovProcess::new(m, states[0].clone()).unwrap();
        assert!(mp.transition_next().is_some());
        let sn = mp.transition_next();
        assert!(
            sn.is_none(),
            "{} is not none because file2 and file3 are both wells",
            sn.unwrap()
        );
    }

    #[test]
    fn test_deserialize_markov_process() {
        use std::fs;
        let p = "./test_data/markov.yaml";
        let mp = MarkovProcess::from_yaml(p);
        assert!(mp.is_ok(), "{}", mp.unwrap_err());
        let mut mp = mp.unwrap();
        while let Some(s) = mp.transition_next() {
            println!("{}", s)
        }
        match mp.current {
            State::Commit => {}
            _ => assert!(false, "Last state should be Commit"),
        }

        let dot_output = mp.to_dot();
        fs::write("markov_from_yaml.dot", dot_output).expect("Unable to write DOT file from YAML");
    }

    #[test]
    fn test_pathing() {
        let p = "./test_data/markov.yaml";
        let mut mp = MarkovProcess::from_yaml(p).unwrap();
        let path = mp.path_steps(10);
        println!("{}", path);
        mp.current = mp.transitions.get_state("issue1").unwrap().clone();
        let path = mp.path_until(&State::Commit);
        assert_eq!(path.iter_count().last().unwrap().0, &State::Commit);
        println!("{}", path);
    }

    #[test]
    fn test_export_dot() {
        use std::fs;
        let mut m = TMatrix::new();
        let states: Vec<State> = vec!["A".into(), "B".into(), "C".into()];
        m.add_states(states.clone());
        m.set_transitions("A", vec![("A", 0.1), ("B", 0.6), ("C", 0.3)]);
        m.set_transitions("B", vec![("B", 0.2), ("C", 0.8)]);
        m.set_transitions("C", vec![("A", 0.5), ("B", 0.5)]);

        let mp = MarkovProcess::new(m, states[0].clone()).unwrap();
        let dot_output = mp.to_dot();

        fs::write("markov.dot", dot_output).expect("Unable to write DOT file");
    }

}
