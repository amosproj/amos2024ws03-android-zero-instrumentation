// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use core::mem;
use aya_ebpf::{maps::RingBuf, macros::{uprobe, map}, programs::{ProbeContext}, EbpfContext, helpers::bpf_ktime_get_ns};
use aya_ebpf::maps::HashMap;
use aya_log_ebpf::error;
use backend_common::{JNICall, JNIMethodName};

const MAP_MAX_ENTRIES: u32 = 100;

#[map(name = "JNI_REF_CALLS" )]
static JNI_REF_CALLS: RingBuf = RingBuf::pinned(MAP_MAX_ENTRIES * mem::size_of::<JNICall>() as u32, 0);

#[map(name = "JNI_REF_PIDS")]
static JNI_REF_PIDS: HashMap<u32, u64> = HashMap::pinned(4096, 0);


fn handle_trace(ctx: ProbeContext, method: JNIMethodName) -> u32 {
    let time_stamp: u64 = unsafe { bpf_ktime_get_ns() };
    let pid = ctx.pid();
    let tid = ctx.tgid();

    if unsafe { JNI_REF_PIDS.get(&pid).is_none() } {
        // don't track calls from this pid
        return 0;
    }

    let mut entry = match JNI_REF_CALLS.reserve::<JNICall>(0) {
        Some(entry) => entry,
        None => {
            error!(&ctx, "could not reserve space in map: JNI_REF_CALLS");
            return 1;
        }
    };

    let entry_mut = entry.as_mut_ptr();

    unsafe {
        (&raw mut (*entry_mut).pid).write(pid);
        (&raw mut (*entry_mut).pid).write(pid);
        (&raw mut (*entry_mut).tid).write(tid);
        (&raw mut (*entry_mut).begin_time_stamp).write(time_stamp);
        (&raw mut (*entry_mut).method_name).write(method);
    }

    entry.submit(0);

    0
}

#[uprobe]
pub fn trace_add_local(ctx: ProbeContext) -> u32 {
    handle_trace(ctx, JNIMethodName::AddLocalRef)
}

#[uprobe]
pub fn trace_del_local(ctx: ProbeContext) -> u32 {
    handle_trace(ctx, JNIMethodName::DeleteLocalRef)
}

#[uprobe]
pub fn trace_add_global(ctx: ProbeContext) -> u32 {
    handle_trace(ctx, JNIMethodName::AddGlobalRef)
}

#[uprobe]
pub fn trace_del_global(ctx: ProbeContext) -> u32 {
    handle_trace(ctx, JNIMethodName::DeleteGlobalRef)
}