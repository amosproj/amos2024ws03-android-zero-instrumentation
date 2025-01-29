// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

pub fn main() {
    let data = garbage_collection::parse::parse(None).unwrap();

    println!("{}", serde_json::to_string(&data).unwrap());
}
