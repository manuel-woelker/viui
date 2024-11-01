use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ComponentNode {
    pub name: String,
    pub children: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub kind: String,
    #[serde(default)]
    pub children: Vec<Node>,
    #[serde(default)]
    pub props: HashMap<String,String>,
    #[serde(default)]
    pub events: HashMap<String,String>,
}