// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya_ebpf::{
    macros::{kprobe, map},
    maps::RingBuf,
    programs::ProbeContext,
    EbpfContext,
};
use backend_common::{KProbeData, KProbeTypes};

#[map(name = "Kprobes")]
pub static KPROBES: RingBuf = RingBuf::with_byte_size(1024, 0);

#[kprobe]
pub fn vfs_write(ctx: ProbeContext) -> Result<(), u32> {
    let pid = ctx.pid();
    let tid = ctx.tgid();

    let data = KProbeData {
        pid,
        tid,
        probe_type: KProbeTypes::VfsWrite,
        ret: false,
    };
    let mut entry = match KPROBES.reserve::<KProbeData>(0) {
        Some(entry) => entry,
        None => return Err(0),
    };

    entry.write(data);
    entry.submit(0);

    Ok(())
}
