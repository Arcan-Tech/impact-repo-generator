use std::{collections::HashMap, hash::Hash, rc::Rc};

use anyhow::bail;
use itertools::Itertools;

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
pub struct Markov {
    states: HashMap<Rc<State>, Vec<Transition>>,
}

impl Markov {
    pub fn new() -> Self {
        Markov {
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

    pub fn next(&self, s: &Rc<State>, p: f32) -> Option<Rc<State>> {
        self.states
            .get(s)
            .expect("state not found")
            .iter()
            .find(|t| t.p <= p) // TODO: this needs to do a cumulative sum
            .map(|t| t.to.clone())
    }
}

#[derive(Debug, Clone)]
pub struct MarkovProcess {
    pub current: Rc<State>,
    m: Markov,
}

impl MarkovProcess {
    pub fn new(m: Markov, starting: Rc<State>) -> anyhow::Result<Self> {
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
        })
    }

    pub fn transition_next(&mut self) -> Rc<State> {
        todo!()
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

#[cfg(test)]
mod tests {
    use crate::stat::markov::MarkovProcess;

    use super::Markov;

    #[test]
    fn test_creation() {
        let mut m = Markov::new();
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

        let mut m = Markov::new();
        let states = vec!["file1", "file2"];
        let states = m.add_states(states);
        m.add_transition(&states[1], (&states[0], 1.1));
        let mp = MarkovProcess::new(m, states[0].clone());
        assert!(mp.is_err(), "{:#?}", mp);
    }
}
