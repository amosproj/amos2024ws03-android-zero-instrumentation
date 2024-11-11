// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use cargo_metadata::MetadataCommand;

pub const AYA_BUILD_EBPF: &str = "AYA_BUILD_EBPF";

pub fn workspace_root() -> PathBuf {
    let metadata = MetadataCommand::new()
        .no_deps()  // You don't need to fetch dependency info here
        .exec()
        .expect("Failed to get cargo metadata");

    metadata.workspace_root.into()
}

pub fn android_launch_path() -> PathBuf {
    let mut base = workspace_root();

    base.push("xtask");
    base.push("scripts");
    base.push("adb_push_and_run.py");

    base
}