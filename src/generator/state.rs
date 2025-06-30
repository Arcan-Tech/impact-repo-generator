use anyhow::bail;
use itertools::Itertools;
use log::warn;
use serde::Deserialize;
use std::hash::Hash;
use std::{collections::HashMap, fmt::Display};

#[derive(Default, Hash, Eq, PartialEq, Clone, Debug, Ord, PartialOrd, Deserialize)]
pub enum State {
    #[default]
    Initial,
    Issue(String),
    Module(String),
    File(String),
    Author(String),
    Commit,
}

impl State {
    pub fn name(&self) -> &str {
        match self {
            State::Initial => "initial_state",
            State::Commit => "commit",
            State::File(name) => name,
            State::Issue(prefix) => prefix,
            State::Module(name) => name,
            State::Author(name) => name,
        }
    }
    pub fn variant_name(&self) -> &str {
        match self {
            State::Initial => "Initial",
            State::Commit => "Commit",
            State::File(_) => "File",
            State::Issue(_) => "Issue",
            State::Module(_) => "Module",
            State::Author(_) => "Author",
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            Self::File(_) => true,
            _ => false,
        }
    }

    pub fn is_author(&self) -> bool {
        match self {
            Self::Author(_) => true,
            _ => false,
        }
    }

    pub fn is_commit(&self) -> bool {
        match self {
            Self::Commit => true,
            _ => false,
        }
    }

    pub fn is_issue(&self) -> bool {
        match self {
            Self::Issue(_) => true,
            _ => false,
        }
    }

    pub fn is_initial(&self) -> bool {
        match self {
            Self::Initial => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Transition {
    pub to: State,
    pub p: f32,
}

pub type Transitions = Vec<Transition>;

#[derive(Debug, Clone, Deserialize)]
pub struct TMatrix {
    matrix: HashMap<State, Transitions>,
}

impl TMatrix {
    pub fn new() -> Self {
        TMatrix {
            matrix: HashMap::new(),
        }
    }

    pub fn set_transitions<S: Into<State>, T: Into<Transition>>(&mut self, s: S, ts: Vec<T>) {
        let s = s.into();
        let mut transitions = ts.into_iter().map(|t| t.into()).collect_vec();
        transitions.sort();
        self.matrix.insert(s, transitions);
    }

    pub fn add_state<S: Into<State>>(&mut self, s: S) {
        self.set_transitions(s, Vec::<(&State, f32)>::new());
    }

    pub fn add_states<S: Into<State>>(&mut self, ss: Vec<S>) {
        ss.into_iter().for_each(|s| self.add_state(s));
    }

    pub fn add_transition<T: Into<Transition>>(&mut self, s: &State, t: T) -> anyhow::Result<()> {
        let t = t.into();
        if let Some(transitions) = self.matrix.get_mut(s) {
            let position = transitions.iter().position(|old| *old == t);
            if let Some(i) = position {
                let _old = std::mem::replace(&mut transitions[i], t);
            } else {
                transitions.push(t);
            }
            transitions.sort();
        } else {
            bail!("Could not find state '{}'", s);
        }
        Ok(())
    }

    pub fn states(&self) -> Vec<&State> {
        self.matrix.keys().collect()
    }

    pub fn iter_transitions_mut(&mut self) -> impl Iterator<Item = &mut Vec<Transition>> {
        self.matrix.iter_mut().map(|(_, v)| v)
    }

    pub fn iter_transitions(&self) -> impl Iterator<Item = &Vec<Transition>> {
        self.matrix.iter().map(|(_, v)| v)
    }

    pub fn contains_state(&self, s: &State) -> bool {
        self.matrix.contains_key(s)
    }

    pub fn get_state(&self, name: &str) -> Option<&State> {
        self.matrix.keys().find(|s| s.name() == name)
    }

    pub fn num_states(&self) -> usize {
        self.matrix.len()
    }

    pub fn num_transitions(&self) -> usize {
        self.matrix.iter().map(|(_, v)| v.len()).sum()
    }

    pub fn as_edge_tuples(&self) -> Vec<(&State, f32, &State)> {
        self.matrix
            .iter()
            .sorted()
            .flat_map(|(from, ts)| ts.iter().map(move |to| (from, to.p, &to.to)))
            .collect()
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        for (s, ts) in &self.matrix {
            let ps = ts.iter().map(|t| t.p).sum::<f32>();
            if ts.len() > 0 && (ps - 1.0).abs() >= 0.00001 {
                bail!(
                    "Transition probabilities of '{}' sum to {} != 1.0",
                    s.name(),
                    ps
                );
            }
        }
        let contains_commit = self.states().iter().any(|s| s.is_commit());
        if !contains_commit {
            bail!("At least one Commit state should be defined")
        }
        let commits_sink = self
            .states()
            .iter()
            .filter(|s| s.is_commit())
            .all(|s| self.matrix.get(s).unwrap().len() == 0);
        if !commits_sink {
            bail!("All Commit states should have no outgoing transitions")
        }
        let contains_author = self.states().iter().any(|s| s.is_author());
        if !contains_author {
            bail!("At least one author should be defined")
        }
        Ok(())
    }

    pub fn next(&self, s: &State, x: f32) -> Option<State> {
        assert!(
            x >= 0.0 && x <= 1.0,
            "Selecting factor must be in the range [0, 1]."
        );
        let mut c = 0.0;
        let mut r = None;
        let ts = self
            .matrix
            .get(s)
            .expect(&format!("state not found: {}", s.name()));
        if ts.is_empty() {
            warn!("No transitions found for state: {}", s);
        }
        for t in ts.iter() {
            c = t.p + c;
            if x <= c {
                let _ = r.insert(t.to.clone());
                break;
            }
        }
        return r;
    }
}

impl From<(&str, f32)> for Transition {
    fn from(value: (&str, f32)) -> Self {
        Self {
            to: State::from(value.0).into(),
            p: value.1,
        }
    }
}
impl From<(&State, f32)> for Transition {
    fn from(value: (&State, f32)) -> Self {
        Transition {
            to: value.0.clone(),
            p: value.1,
        }
    }
}
impl From<&str> for State {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}
impl From<String> for State {
    fn from(value: String) -> Self {
        State::File(value)
    }
}
impl Hash for Transition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to.hash(state);
    }
}
impl Eq for Transition {}
impl PartialEq for Transition {
    fn eq(&self, other: &Self) -> bool {
        self.to.eq(&other.to)
    }
}
impl PartialOrd for Transition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.p.partial_cmp(&other.p)
    }
}
impl Ord for Transition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.p.total_cmp(&other.p)
    }
}
impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.variant_name(), self.name())
    }
}

#[cfg(test)]
mod tests {

    use super::TMatrix;

    #[test]
    fn test_creation() {
        let mut m = TMatrix::new();
        let states = vec!["file1".into(), "file3".into()];
        m.add_states(states.clone());
        m.add_transition(&states[0], (&states[1], 0.65)).unwrap();
        m.add_transition(&states[0], (&states[0], 0.35)).unwrap();
        m.add_transition(&states[0], (&states[0], 0.35)).unwrap();
        println!("{:?}", m);
        assert_eq!(m.matrix[&states[0]].len(), 2);

        assert_eq!(
            m.matrix[&states[0]],
            vec![(&states[0], 0.35).into(), (&states[1], 0.65).into()]
        );
        assert_ne!(
            m.matrix[&states[0]],
            vec![(&states[1], 0.65).into(), (&states[0], 0.35).into()]
        );
    }

    #[test]
    fn test_generation() {
        let mut m = TMatrix::new();
        let states = vec!["file1".into(), "file2".into()];
        m.add_states(states.clone());
        let s0 = &states[0];
        m.add_transition(s0, (&states[1], 0.65)).unwrap();
        m.add_transition(s0, (&states[0], 0.35)).unwrap();
        let n0 = m.next(s0, 0.30);
        assert!(n0.is_some());
        assert_eq!(n0.unwrap(), states[0]);
        let n0 = m.next(s0, 0.50);
        assert_eq!(n0.unwrap(), states[1]);
        let n0 = m.next(s0, 1.0);
        assert_eq!(n0.unwrap(), states[1]);
    }
}
