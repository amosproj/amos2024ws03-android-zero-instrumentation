// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya_ebpf::{macros::{tracepoint, map}, maps::{HashMap, RingBuf}, programs::{TracePointContext}, EbpfContext, helpers::gen::bpf_ktime_get_ns};
use aya_ebpf::maps::Array;
use aya_log_ebpf::error;
use backend_common::{BlockingMainCall, MainBlockingCause};
use backend_common::MainBlockingCause::Futex;

#[map(name = "BLOCKING_MAIN_EVENTS")]
pub static BLOCKING_MAIN_EVENTS: RingBuf = RingBuf::pinned(2048, 0);

#[map(name = "BLOCKING_MAIN_PIDS")]
static BLOCKING_MAIN_PIDS: HashMap<u32, u64> = HashMap::pinned(128, 0);

#[map(name = "MAIN_THREAD_TIMESTAMP")]
static MAIN_THREAD_TIMESTAMP: Array<u64> = Array::pinned(1, 0);


fn handle_enter(pid: u32) -> u32 {
    let begin_time_stamp: u64 = unsafe { bpf_ktime_get_ns() };

    if unsafe { BLOCKING_MAIN_PIDS.get(&pid) }.is_none() {
        return 1; // ignore pid as it is not the main thread
    }

    let timestamp_mut_point = match MAIN_THREAD_TIMESTAMP.get_ptr_mut(0) {
        Some(ts) => ts,
        None => return 1,
    };

    unsafe {
        *timestamp_mut_point = begin_time_stamp;
    }

    0
}

fn handle_exit(blocking_cause: MainBlockingCause, pid: u32, tid: u32) -> Result<(), ()> {
    let end_time = unsafe { bpf_ktime_get_ns() };

    let time_threshold_ns = match unsafe { BLOCKING_MAIN_PIDS.get(&pid) } {
        Some(v) => *v,
        None => return Err(()), // ignore pid as it is not the main thread
    };

    let time_threshold_ns = 0;

    let begin_time = match MAIN_THREAD_TIMESTAMP.get(0) {
        Some(v) => *v,
        None => return Err(()),
    };
    let duration_nano_sec = end_time - begin_time;

    if duration_nano_sec < time_threshold_ns {
        // call is not declared blocking, discard
        return Ok(());
    }

    let mut entry = match BLOCKING_MAIN_EVENTS.reserve::<BlockingMainCall>(0) {
        Some(entry) => entry,
        None => return Err(()),
    };

    let entry_mut = entry.as_mut_ptr();

    unsafe {
        (&raw mut (*entry_mut).pid).write(pid);
        (&raw mut (*entry_mut).tid).write(tid);
        (&raw mut (*entry_mut).begin_time_stamp).write(begin_time);
        (&raw mut (*entry_mut).blocking_cause).write(blocking_cause);
        (&raw mut (*entry_mut).duration_nano_sec).write(duration_nano_sec);
    }

    entry.submit(0);

    Ok(())
}

#[tracepoint]
pub fn sys_enter_futex(ctx: TracePointContext) -> u32 {
    handle_enter(ctx.pid())
}


#[tracepoint]
pub fn sys_exit_futex(ctx: TracePointContext) -> u32 {
    match handle_exit(Futex, ctx.pid(), ctx.tgid()) {
        Ok(_) => 0,
        Err(_) => {
            error!(&ctx, "could not reserve space in map: BLOCKING_MAIN_EVENTS");
            0
        }
    }
}


