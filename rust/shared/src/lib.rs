// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

pub mod counter {
    tonic::include_proto!("com.example.counter");
}

pub mod ziofa {
    tonic::include_proto!("ziofa");
}

pub mod config{
    tonic::include_proto!("config");
}