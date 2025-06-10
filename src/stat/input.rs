use std::collections::HashMap;

use serde::Deserialize;

use super::markov::{State, Transitions};

#[derive(Deserialize, Clone)]
pub struct MarkovChain {
    pub transitions: HashMap<State, Transitions>,
}
