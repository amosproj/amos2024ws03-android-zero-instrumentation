#![no_std]

// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

// This file exists to enable the library target.

mod vfs_write;
pub mod sys_write;

pub use vfs_write::{vfs_write, VFS_WRITE_MAP};
pub use sys_write::{SYS_WRITE_MAP};
