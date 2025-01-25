// SPDX-FileCopyrightText: 2025 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya_ebpf::{helpers::gen::bpf_ktime_get_ns, macros::map, maps::RingBuf, programs::TracePointContext, EbpfContext};
use aya_ebpf::maps::HashMap;
use aya_log_ebpf::{error};
use backend_common::{SysFdAction, SysFdActionCall};

#[map(name = "SYS_FDTRACKING_PIDS")]
static SYS_FDTRACKING_PIDS: HashMap<u32, u64> = HashMap::pinned(4096, 0);

#[map(name = "SYS_FDTRACKING_EVENTS")]
pub static SYS_FDTRACKING_EVENTS: RingBuf = RingBuf::pinned(1024, 0);

// Disclaimer:
// We have to swap here, because BPF_PROG_TEST_RUN does not support Tracepoints
// For testing we can set the prog-test flag and interpret it as TracepointContext, because we can set whatever we want
// For an example see backend/daemon/src/prog_test_run.rs

#[cfg(feature = "prog-test")]
type Arg = aya_ebpf::programs::RawTracePointContext;

#[cfg(not(feature = "prog-test"))]
type Arg = aya_ebpf::programs::TracePointContext;

#[cfg_attr(feature = "prog-test", aya_ebpf::macros::raw_tracepoint)]
#[cfg_attr(not(feature = "prog-test"), aya_ebpf::macros::tracepoint)]
pub fn sys_create_fd(ctx: Arg) -> u32 {
    let ctx = TracePointContext::new(ctx.as_ptr());
    handle_fd_action(ctx, SysFdAction::Created)
}


#[cfg_attr(feature = "prog-test", aya_ebpf::macros::raw_tracepoint)]
#[cfg_attr(not(feature = "prog-test"), aya_ebpf::macros::tracepoint)]
pub fn sys_destroy_fd(ctx: Arg) -> u32 {
    let ctx = TracePointContext::new(ctx.as_ptr());
    handle_fd_action(ctx, SysFdAction::Destroyed)
}


fn handle_fd_action(ctx: TracePointContext, fd_action: SysFdAction) -> u32 {
    let pid = ctx.pid();

    let tid = ctx.tgid();

    let return_value: i64;
    let time_stamp: u64;

    unsafe {
        return_value = match ctx.read_at(16) {
            Ok(arg) => arg,
            Err(_) => return 1,
        };
        time_stamp = bpf_ktime_get_ns();
    }

    if return_value == -1 {
        return 1; // the syscall was unsuccessful -> don't evaluate the call
    }

    let mut entry = match SYS_FDTRACKING_EVENTS.reserve::<SysFdActionCall>(0) {
        Some(entry) => entry,
        None => {
            error!(&ctx, "could not reserve space in map: SYS_FDTRACKING_EVENTS");
            return 1;
        }
    };

    let entry_mut = entry.as_mut_ptr();

    unsafe {
        (&raw mut (*entry_mut).pid).write(pid);
        (&raw mut (*entry_mut).tid).write(tid);
        (&raw mut (*entry_mut).time_stamp).write(time_stamp);
        (&raw mut (*entry_mut).fd_action).write(fd_action);
    }

    entry.submit(0);

    0
}
