use std::{collections::HashMap, fmt::Display, fs, hash::Hash, path::Path};

use anyhow::bail;
use itertools::Itertools;
use rand::rng;
use rand_distr::{Distribution, Uniform};
use serde::Deserialize;

#[derive(Hash, Default, Eq, PartialEq, Clone, Debug, Deserialize)]
pub enum State {
    #[default]
    Initial,
    File(String),
    Author(String),
    Issue(String),
    Module(String),
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
            Self::Issue => true,
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
        let contains_commit = self.states().iter().any(|s| matches!(**s, State::Commit));
        if !contains_commit {
            bail!("At least one Commit state should be defined")
        }
        let commits_sink = self
            .states()
            .iter()
            .filter(|s| matches!(**s, State::Commit))
            .all(|s| self.matrix.get(s).unwrap().len() == 0);
        if !commits_sink {
            bail!("All Commit states should have no outgoing transitions")
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
        for t in self
            .matrix
            .get(s)
            .expect(&format!("state not found: {}", s.name()))
            .iter()
        {
            c = t.p + c;
            if x <= c {
                let _ = r.insert(t.to.clone());
                break;
            }
        }
        return r;
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarkovProcess {
    current: State,
    starting: State,
    transitions: TMatrix,

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
            transitions: m,
            d: default_uniform(),
        })
    }

    pub fn from_yaml<P: AsRef<Path>>(p: P) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(p)?;
        let mut mp: Self = serde_yaml::from_str(&contents)?;
        let mut to_add = Vec::new();
        for (_, ts) in mp.transitions.matrix.iter_mut() {
            ts.sort();
        }
        for (_, ts) in mp.transitions.matrix.iter() {
            for s in ts {
                if !mp.transitions.matrix.contains_key(&s.to) {
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

    pub fn transition_next(&mut self) -> Option<&State> {
        let x = self.d.sample(&mut rng());
        if let Some(next) = self.transitions.next(&self.current, x) {
            self.current = next;
            return Some(&self.current);
        };
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
            }
        }
        path
    }

    pub fn reset(&mut self) {
        self.current = self.starting.clone();
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
    use crate::stat::markov::{MarkovProcess, State};

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

        let mp = MarkovProcess::new(m, states[0].clone());
        assert!(mp.is_ok(), "{:#?}", mp);
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
    }

    #[test]
    fn test_pathing() {
        let p = "./test_data/markov.yaml";
        let mut mp = MarkovProcess::from_yaml(p).unwrap();
        let path = mp.path_steps(10);
        println!("{}", path);
        mp.current = State::Issue("issue1".to_string());
        let path = mp.path_until(&State::Commit);
        assert_eq!(path.iter_count().last().unwrap().0, &State::Commit);
        println!("{}", path);
    }
}
