// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya_ebpf::{macros::{tracepoint, map}, maps::{HashMap, RingBuf}, programs::{TracePointContext}, EbpfContext, helpers::gen::bpf_ktime_get_ns};
use aya_log_ebpf::error;
use backend_common::{generate_id, SysSendmsgCall};

#[map(name = "SYS_SENDMSG_EVENTS")]
pub static SYS_SENDMSG_EVENTS: RingBuf = RingBuf::with_byte_size(1024, 0);

#[map(name = "SYS_SENDMSG_PIDS")]
static SYS_SENDMSG_PIDS: HashMap<u32, u64> = HashMap::with_max_entries(4096, 0);

#[map(name = "SYS_SENDMSG_TIMESTAMPS")]
static SYS_SENDMSG_TIMESTAMPS: HashMap<u64, SysSendmsgIntern> = HashMap::with_max_entries(1024, 0);


struct SysSendmsgIntern {
    begin_time_stamp: u64,
    fd: u64,
}

#[tracepoint]
pub fn sys_enter_sendmsg(ctx: TracePointContext) -> u32 {
    let pid = ctx.pid();
    let id = generate_id(pid, ctx.tgid());

    if unsafe { SYS_SENDMSG_PIDS.get(&pid).is_none() } {
        return 0;
    }

    let begin_time_stamp;
    let fd: u64;
    unsafe {
        fd = match ctx.read_at(16) {
            Ok(arg) => arg,
            Err(_) => return 1,
        };
        begin_time_stamp = bpf_ktime_get_ns();
    }

    let data: SysSendmsgIntern = SysSendmsgIntern {begin_time_stamp, fd};

    match SYS_SENDMSG_TIMESTAMPS.insert(&id, &data, 0) {
            Ok(_) => 0,
            Err(_) => 1,
    }
}


#[tracepoint]
pub fn sys_exit_sendmsg(ctx: TracePointContext) -> u32 {
    let end_time = unsafe { bpf_ktime_get_ns() };
    let pid = ctx.pid();

    let duration_threshold_nano_sec = match unsafe { SYS_SENDMSG_PIDS.get(&pid) } {
        None => return 0, // pid should not be tracked
        Some(duration) => duration,
    };


    let tgid = ctx.tgid();
    let call_id = generate_id(pid, tgid);
    let data = match unsafe { SYS_SENDMSG_TIMESTAMPS.get(&call_id) } {
        None => {return 1}
        Some(entry) => {entry}
    };
    let _ = SYS_SENDMSG_TIMESTAMPS.remove(&call_id);

    let duration_nano_sec = end_time - data.begin_time_stamp;

    if duration_nano_sec < *duration_threshold_nano_sec {
        return 0;
    }

    let result_data = SysSendmsgCall::new(pid, tgid, data.begin_time_stamp, data.fd, duration_nano_sec);

    let mut entry = match SYS_SENDMSG_EVENTS.reserve::<SysSendmsgCall>(0) {
        Some(entry) => entry,
        None => {
            error!(&ctx, "could not reserve space in SYS_SENDMSG_MAP");
            return 1;
        }
    };

    entry.write(result_data);
    entry.submit(0);


    0
}
