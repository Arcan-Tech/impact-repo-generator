use std::{collections::HashMap, fmt::Display, hash::Hash, rc::Rc};

use anyhow::bail;
use itertools::Itertools;
use rand::rng;
use rand_distr::{Distribution, Uniform};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct State {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Transition {
    pub to: Rc<State>,
    pub p: f32,
}

#[derive(Debug, Clone)]
pub struct TMatrix {
    states: HashMap<Rc<State>, Vec<Transition>>,
}

impl TMatrix {
    pub fn new() -> Self {
        TMatrix {
            states: HashMap::new(),
        }
    }

    pub fn add_states<S: Into<State>>(&mut self, s: Vec<S>) -> Vec<Rc<State>> {
        s.into_iter().map(|s| self.add_state(s)).collect_vec()
    }

    pub fn add_state<S: Into<State>>(&mut self, s: S) -> Rc<State> {
        let s = Rc::new(s.into());
        self.states.insert(s.clone(), Vec::new());
        return s;
    }

    pub fn set_transitions<S: Into<State>, T: Into<Transition>>(&mut self, s: S, ts: Vec<T>) {
        let s = s.into().into();
        let mut transitions = ts.into_iter().map(|t| t.into()).collect_vec();
        transitions.sort();
        self.states.insert(s, transitions);
    }

    pub fn add_transition<T: Into<Transition>>(&mut self, s: &Rc<State>, t: T) {
        let t = t.into();
        let transitions = self.states.get_mut(s).expect("state not found");
        let position = transitions.iter().position(|old| *old == t);
        if let Some(i) = position {
            let _old = std::mem::replace(&mut transitions[i], t);
        } else {
            transitions.push(t);
        }
        transitions.sort();
    }

    pub fn next(&self, s: &Rc<State>, x: f32) -> Option<Rc<State>> {
        assert!(
            x >= 0.0 && x <= 1.0,
            "Selecting factor must be in the range [0, 1]."
        );
        let mut c = 0.0;
        let mut r = None;
        for t in self.states.get(s).expect("state not found").iter() {
            c = t.p + c;
            if x <= c {
                let _ = r.insert(t.to.clone());
                break;
            }
        }
        return r;
    }
}

#[derive(Debug, Clone)]
pub struct MarkovProcess {
    pub current: Rc<State>,
    m: TMatrix,
    d: Uniform<f32>,
}

impl MarkovProcess {
    pub fn new(m: TMatrix, starting: Rc<State>) -> anyhow::Result<Self> {
        for (s, ts) in &m.states {
            let ps = ts.iter().map(|t| t.p).sum::<f32>();
            if ts.len() > 0 && (ps - 1.0).abs() >= 0.00001 {
                dbg!(ts);
                bail!(
                    "Transition probabilities of '{}' sum to {} > 1.0",
                    s.name,
                    ps
                );
            }
        }

        Ok(Self {
            current: starting,
            m,
            d: Uniform::new(0.0, 1.0)?,
        })
    }

    pub fn transition_next(&mut self) -> Option<Rc<State>> {
        let x = self.d.sample(&mut rng());
        let next = self.m.next(&self.current, x);
        if let Some(next) = next.clone() {
            self.current = next;
        }
        next
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
impl From<(&Rc<State>, f32)> for Transition {
    fn from(value: (&Rc<State>, f32)) -> Self {
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
        State { name: value }
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
        write!(f, "<{}>", self.name)
    }
}
#[cfg(test)]
mod tests {
    use crate::stat::markov::MarkovProcess;

    use super::TMatrix;

    #[test]
    fn test_creation() {
        let mut m = TMatrix::new();
        let states = vec!["file1", "file2"];
        let states = m.add_states(states);
        m.add_transition(&states[0], (&states[1], 0.65));
        m.add_transition(&states[0], (&states[0], 0.35));
        m.add_transition(&states[0], (&states[0], 0.35));
        assert_eq!(m.states[&states[0]].len(), 2);

        assert_eq!(
            m.states[&states[0]],
            vec![(&states[0], 0.35).into(), (&states[1], 0.65).into()]
        );
        assert_ne!(
            m.states[&states[0]],
            vec![(&states[1], 0.65).into(), (&states[0], 0.35).into()]
        );

        let mp = MarkovProcess::new(m, states[0].clone());
        assert!(mp.is_ok(), "{:#?}", mp);
    }

    #[test]
    fn test_generation() {
        let mut m = TMatrix::new();
        let states = m.add_states(vec!["file1", "file2"]);
        let s0 = &states[0];
        m.add_transition(s0, (&states[1], 0.65));
        m.add_transition(s0, (&states[0], 0.35));
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
        let states = vec!["file1", "file2", "file3"];
        let states = m.add_states(states);
        m.set_transitions("file1", vec![("file1", 0.5), ("file2", 0.5)]);
        m.set_transitions("file2", vec![("file1", 0.80), ("file2", 0.2)]);
        let mut mp = MarkovProcess::new(m, states[0].clone()).unwrap();
        for _i in 0..100 {
            let sn = mp.transition_next();
            assert!(sn.is_some());
            let name = &sn.unwrap().name;
            assert!(name == "file1" || name == "file2");
        }

        let mut m = TMatrix::new();
        let states = m.add_states(vec!["file1", "file2", "file3"]);
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
}
