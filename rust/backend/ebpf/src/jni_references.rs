// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use core::mem;
use aya_ebpf::{maps::RingBuf, macros::{uprobe, map}, programs::{ProbeContext}, EbpfContext, helpers::bpf_ktime_get_ns};
use aya_log_ebpf::error;
use backend_common::{JNICall, JNIMethodName};

const MAP_MAX_ENTRIES: u32 = 100;

#[map(name = "JNI_CALLS" )]
static JNI_CALLS: RingBuf = RingBuf::with_byte_size(MAP_MAX_ENTRIES * mem::size_of::<JNICall>() as u32, 0);

#[uprobe]
pub fn trace_add_local(ctx: ProbeContext) -> u32 {
    let time_stamp: u64 = unsafe { bpf_ktime_get_ns() };
    let pid = ctx.pid();
    let tid = ctx.tgid();

    let call = JNICall {
        pid: pid,
        tid: tid,
        begin_time_stamp: time_stamp,
        method_name: JNIMethodName::AddLocalRef,
    };

    let mut entry = match JNI_CALLS.reserve::<JNICall>(0) {
        Some(entry) => entry,
        None => {
            error!(&ctx, "could not reserve space in map: JNI_CALLS");
            return 1;
        }
    };

    entry.write(call);
    entry.submit(0);

    0
}

#[uprobe]
pub fn trace_del_local(ctx: ProbeContext) -> u32 {
    let time_stamp: u64 = unsafe { bpf_ktime_get_ns() };
    let pid = ctx.pid();
    let tid = ctx.tgid();

    let call = JNICall {
        pid: pid,
        tid: tid,
        begin_time_stamp: time_stamp,
        method_name: JNIMethodName::DeleteLocalRef,
    };

    let mut entry = match JNI_CALLS.reserve::<JNICall>(0) {
        Some(entry) => entry,
        None => {
            error!(&ctx, "could not reserve space in map: JNI_CALLS");
            return 1;
        }
    };

    entry.write(call);
    entry.submit(0);

    0
}

#[uprobe]
pub fn trace_add_global(ctx: ProbeContext) -> u32 {
    let time_stamp: u64 = unsafe { bpf_ktime_get_ns() };
    let pid = ctx.pid();
    let tid = ctx.tgid();

    let call = JNICall {
        pid: pid,
        tid: tid,
        begin_time_stamp: time_stamp,
        method_name: JNIMethodName::AddGlobalRef,
    };

    let mut entry = match JNI_CALLS.reserve::<JNICall>(0) {
        Some(entry) => entry,
        None => {
            error!(&ctx, "could not reserve space in map: JNI_CALLS");
            return 1;
        }
    };

    entry.write(call);
    entry.submit(0);

    0
}

#[uprobe]
pub fn trace_del_global(ctx: ProbeContext) -> u32 {
    let time_stamp: u64 = unsafe { bpf_ktime_get_ns() };
    let pid = ctx.pid();
    let tid = ctx.tgid();

    let call = JNICall {
        pid: pid,
        tid: tid,
        begin_time_stamp: time_stamp,
        method_name: JNIMethodName::DeleteGlobalRef,
    };

    let mut entry = match JNI_CALLS.reserve::<JNICall>(0) {
        Some(entry) => entry,
        None => {
            error!(&ctx, "could not reserve space in map: JNI_CALLS");
            return 1;
        }
    };

    entry.write(call);
    entry.submit(0);

    0
}