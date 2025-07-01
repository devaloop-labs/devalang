use serde::{ Deserialize, Serialize };

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Duration {
    Number(f32),
    Identifier(String),
    Auto,
}
