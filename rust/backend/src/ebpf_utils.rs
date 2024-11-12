// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::sync::MutexGuard;
use aya::programs::kprobe::KProbeLinkId;
use aya::programs::uprobe::UProbeLinkId;
use aya::programs::UProbe;
use aya::{include_bytes_aligned, programs::KProbe, Ebpf};
use libc::{pid_t};
use shared::config::ebpf_entry::UprobeConfig;
use crate::configuration::load_from_file;
pub enum ProbeID {
    KProbeID(KProbeLinkId),
    UProbeID(UProbeLinkId),
}
fn load_function(ebpf: &mut Ebpf, hash_map: &mut HashMap<String, ProbeID>,
                 probe_type: Option<UprobeConfig>, func: &str, hook: &str) {
    /* examples:
     * func: "kprobetcp"
     * hook: "tcp_connect"
     */

    // TODO: Error checking

    match probe_type {
        // UPROBE
        Some(UprobeConfig {
            offset,
            target,
            pid,
        }) => {
            // get ebpf program
            let program: &mut UProbe = ebpf.program_mut(func).unwrap().try_into().unwrap();

            // load ebpf program
            program.load().unwrap();

            // attach ebpf program and insert its ProbeID into the hash map
            hash_map.insert(func.to_string(),
                            ProbeID::UProbeID(program.attach(Some(hook), offset, target, pid).unwrap()));
        }
        // KPROBE/KRETPROBE
        None => {
            let program: &mut KProbe = ebpf.program_mut(func).unwrap().try_into().unwrap();
            program.load().unwrap();
            hash_map.insert(func.to_string(),
                            ProbeID::KProbeID(program.attach(hook, 0).unwrap()));
        }
    }
}

fn unload_function(ebpf: &mut Ebpf, hash_map: &mut HashMap<String, ProbeID>, func: &str) {
    // get ProbeID and remove it from hash map
    let probe = hash_map.remove(func).unwrap();

    match probe{
        ProbeID::UProbeID(program) => {
            // get ebpf program
            let program: &mut UProbe = ebpf.program_mut(func).unwrap().try_into().unwrap();

            // unload ebpf program
            program.unload().unwrap();
        },
        ProbeID::KProbeID(program) => {
            let program: &mut KProbe = ebpf.program_mut(func).unwrap().try_into().unwrap();
            program.unload().unwrap();
        }
    }
}

pub fn update_from_config(ebpf: &mut Ebpf, config_path: &str, loaded_functions: &mut HashMap<String, ProbeID>) {
    let entries = load_from_file(config_path).unwrap().entries;

    for entry in entries {
        if entry.attach {
            match loaded_functions.get(entry.ebpf_name.as_str()) {
                Some(res) => {}
                None => {
                    load_function(ebpf, loaded_functions, entry.uprobe_info, entry.ebpf_name.as_str(), entry.hook.as_str())
                }
            }
        } else {
            match loaded_functions.get(entry.ebpf_name.as_str()) {
                Some(res) => {
                    unload_function(ebpf, loaded_functions, entry.ebpf_name.as_str());
                },
                None => {}
            }
        }
    }
}