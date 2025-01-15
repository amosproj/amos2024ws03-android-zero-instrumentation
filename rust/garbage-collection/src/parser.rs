// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use garbage_collection::parse::parse;

pub fn main() {
    let data = parse(None).unwrap();
    
    println!("{}", serde_json::to_string(&data).unwrap());
}
