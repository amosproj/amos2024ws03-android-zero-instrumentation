// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Execution {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub root: bool
}

#[derive(Serialize, Deserialize)]
pub struct HostSpec {
    pub root: bool,
    pub env: HashMap<String, String>,
    pub args: Vec<String>,
    pub runner_path: String,
}

#[derive(Serialize, Deserialize)]
pub enum Command {
    Exit
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Log {
    Stderr(String),
    Stdout(String),
    InternalError(String)
}