// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya_ebpf::{macros::{tracepoint, map}, maps::{HashMap, RingBuf}, programs::{TracePointContext}, EbpfContext, helpers::gen::bpf_ktime_get_ns, bpf_printk};
use backend_common::{generate_id, SysSendmsgCall};

#[map(name = "SYS_SENDMSG_MAP")]
pub static SYS_SENDMSG_MAP: RingBuf = RingBuf::with_byte_size(1024, 0);


#[map]
static SYS_SENDMSG_TIMESTAMPS: HashMap<u64, SysSendmsgIntern> = HashMap::with_max_entries(1024, 0);


struct SysSendmsgIntern {
    begin_time_stamp: u64,
    fd: i32,
}



#[tracepoint]
pub fn sys_enter_sendmsg(ctx: TracePointContext) -> u32 {
    let id = generate_id(ctx.pid(), ctx.tgid());

    let begin_time_stamp;
    let fd: i32;
    unsafe {
        begin_time_stamp = bpf_ktime_get_ns();
        fd = match ctx.read_at(16) {
            Ok(arg) => arg,
            Err(_) => return 1,
        };
    }

    let data: SysSendmsgIntern = SysSendmsgIntern {begin_time_stamp, fd};

    match SYS_SENDMSG_TIMESTAMPS.insert(&id, &data, 0) {
            Ok(_) => 0,
            Err(_) => 1,
    }
}


#[tracepoint]
pub fn sys_exit_sendmsg(ctx: TracePointContext) -> u32 {
    let pid = ctx.pid();
    let tgid = ctx.tgid();
    let call_id = generate_id(pid, tgid);
    let data = match unsafe { SYS_SENDMSG_TIMESTAMPS.get(&call_id) } {
        None => {return 1}
        Some(entry) => {entry}
    };


    let result_data = SysSendmsgCall::new(pid, tgid, data.begin_time_stamp, data.fd);

    let mut entry = match SYS_SENDMSG_MAP.reserve::<SysSendmsgCall>(0) {
        Some(entry) => entry,
        None => return 1,
    };

    entry.write(result_data);
    entry.submit(0);


    0
}