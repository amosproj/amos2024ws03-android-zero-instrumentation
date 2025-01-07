// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya_ebpf::{macros::{tracepoint, map}, maps::{RingBuf}, programs::{TracePointContext}, EbpfContext, helpers::gen::bpf_ktime_get_ns};
use aya_ebpf::maps::HashMap;
use aya_log_ebpf::error;
use backend_common::{SysSigquitCall};

#[map(name = "SYS_SIGQUIT_PIDS")]
static SYS_SIGQUIT_PIDS: HashMap<u32, u64> = HashMap::pinned(4096, 0);

#[map(name = "SYS_SIGQUIT_EVENTS")]
pub static SYS_SIGQUIT_EVENTS: RingBuf = RingBuf::pinned(1024, 0);

#[tracepoint]
pub fn sys_sigquit(ctx: TracePointContext) -> u32 {
    let pid = ctx.pid();

    if unsafe { SYS_SIGQUIT_PIDS.get(&pid).is_none() } {
        // ignore signals from this pid
        return 0;
    }

    let tid = ctx.tgid();

    let time_stamp: u64;
    let target_pid: u64;
    let signal: u64;


    unsafe {
        time_stamp = bpf_ktime_get_ns();
        target_pid = match ctx.read_at(16) {
            Ok(arg) => arg,
            Err(_) => return 1,
        };
        signal = match ctx.read_at(24) {
            Ok(arg) => arg,
            Err(_) => return 1,
        };
    }

    if signal != 3u64 { // libc::SIGQUIT
        // discard event
        return 0;
    }

    let mut entry = match SYS_SIGQUIT_EVENTS.reserve::<SysSigquitCall>(0) {
        Some(entry) => entry,
        None => {
            error!(&ctx, "could not reserve space in map: SYS_SIGQUIT_CALLS");
            return 1;
        }
    };

    let entry_mut = entry.as_mut_ptr();

    unsafe {
        (&raw mut (*entry_mut).pid).write(pid);
        (&raw mut (*entry_mut).tid).write(tid);
        (&raw mut (*entry_mut).time_stamp).write(time_stamp);
        (&raw mut (*entry_mut).target_pid).write(target_pid);
    }

    entry.submit(0);

    0
}
