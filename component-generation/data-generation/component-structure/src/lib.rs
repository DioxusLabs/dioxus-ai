use serde::{Deserialize, Serialize};

mod prompts;
pub use prompts::*;

#[derive(Deserialize, Serialize, Debug, Eq, Hash, PartialEq)]
pub struct Train {
    pub prompt: String,
    pub response: Chat,
}

#[derive(Deserialize, Serialize, Debug, Eq, Hash, PartialEq)]
pub struct Chat {
    #[serde(default)]
    pub model: String,
    pub choices: Vec<Choice>,
}

#[derive(Deserialize, Serialize, Debug, Eq, Hash, PartialEq)]
pub struct Choice {
    pub message: Message,
}

#[derive(Deserialize, Serialize, Debug, Eq, Hash, PartialEq)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Logprobs {
    #[serde(default)]
    pub tokens: Vec<f64>,
    #[serde(default)]
    pub token_logprobs: Vec<f64>,
    #[serde(default)]
    pub top_logprobs: Vec<f64>,
    #[serde(default)]
    pub text_offset: Vec<usize>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Usage {
    #[serde(default)]
    pub prompt_tokens: usize,
    #[serde(default)]
    pub completion_tokens: usize,
    #[serde(default)]
    pub total_tokens: usize,
    #[serde(default)]
    pub prompt_time: f64,
    #[serde(default)]
    pub completion_time: f64,
    #[serde(default)]
    pub total_time: f64,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct XGroq {
    #[serde(default)]
    pub id: String,
}
