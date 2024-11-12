// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use aya::programs::kprobe::KProbeLinkId;
use aya::programs::uprobe::UProbeLinkId;
use aya::programs::UProbe;
use aya::{include_bytes_aligned, programs::KProbe, Ebpf};
use libc::{pid_t};
use crate::configuration::load_from_file;

enum ProbeType{
    UPROBE {
        offset: u64,
        target: String,
        pid: Option<pid_t>,
    },
    KPROBE,
    KRETPROBE,
}

pub enum ProbeID {
    KProbeID(KProbeLinkId),
    UProbeID(UProbeLinkId),
}
fn load_function(ebpf: &mut Ebpf, hash_map: &mut HashMap<String, ProbeID>,
                 probe_type: ProbeType, func: &str, hook: &str) {
    /* examples:
     * func: "kprobetcp"
     * hook: "tcp_connect"
     */

    // TODO: Error checking

    match probe_type {
        ProbeType::UPROBE {
            offset,
            target,
            pid,
        } => {
            // get ebpf program
            let program: &mut UProbe = ebpf.program_mut(func).unwrap().try_into().unwrap();

            // load ebpf program
            program.load().unwrap();

            // attach ebpf program and insert its ProbeID into the hash map
            hash_map.insert(func.to_string(),
                            ProbeID::UProbeID(program.attach(Some(hook), offset, target, pid).unwrap()));
        }
        ProbeType::KPROBE | ProbeType::KRETPROBE => {
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

// TODO
pub fn update_from_config(config_path: &str, loaded_functions: HashMap<String, ProbeID>) {
    let config = load_from_file(config_path).unwrap().entries;
}