// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya::programs::kprobe::KProbeLinkId;
use aya::programs::uprobe::UProbeLinkId;
use aya::programs::UProbe;
use aya::{include_bytes_aligned, programs::KProbe, Ebpf};
use libc::{pid_t};
use std::path::Path;

enum ProbeType{
    UPROBE {
        offset: u64,
        target: String,
        pid: Option<pid_t>,
    },
    KPROBE,
    KRETPROBE,
}

enum ProbeID {
    KProbeID(KProbeLinkId),
    UProbeID(UProbeLinkId),
}
fn load_function(probe_type: ProbeType, func: &str, hook: &str) -> ProbeID {
    /* examples:
     * func: "kprobetcp"
     * hook: "tcp_connect"
     */

    // TODO: Error checking
    let mut bpf = Ebpf::load(
        include_bytes_aligned!(concat!(env!("OUT_DIR"), "/example"))).unwrap();

    match probe_type {
        ProbeType::UPROBE {
            offset,
            target,
            pid,
        } => {
            let program: &mut UProbe = bpf.program_mut(func).unwrap().try_into().unwrap();
            program.load().unwrap();
            ProbeID::UProbeID(program.attach(Some(hook), offset, target, pid).unwrap())
        }
        ProbeType::KPROBE | ProbeType::KRETPROBE => {
            let program: &mut KProbe = bpf.program_mut(func).unwrap().try_into().unwrap();
            program.load().unwrap();
            ProbeID::KProbeID(program.attach(hook, 0).unwrap())
        }
    }
}
