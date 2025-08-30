use devalang_types::BankInfo;
use serde::Deserialize;

pub mod api;
pub mod commands;

#[derive(Debug, Deserialize)]
pub struct BankList {
    pub bank: Vec<BankInfo>,
}

#[derive(Debug, Deserialize)]
pub struct BankInfoFetched {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub latest_version: String,
}

#[derive(Debug, Deserialize)]
pub struct BankVersion {
    pub version: String,
}

pub use commands::{
    handle_bank_available_command, handle_bank_info_command, handle_bank_list_command,
    handle_remove_bank_command, handle_update_bank_command,
};
