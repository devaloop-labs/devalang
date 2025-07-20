use serde::{ Deserialize, Serialize };

#[derive(Debug, Deserialize)]
pub struct BankInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

#[derive(Debug, Deserialize)]
pub struct BankFile {
    pub bank: BankInfo,
    pub triggers: Option<Vec<BankTrigger>>,
}

#[derive(Debug, Deserialize)]
pub struct BankTrigger {
    pub name: String,
    pub path: String,
}
