// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya_ebpf::{
    macros::{btf_tracepoint, map},
    maps::{HashMap, RingBuf},
    programs::{BtfTracePointContext},
    EbpfContext,
    helpers::gen::bpf_ktime_get_ns,
};
use backend_common::{generate_id, SysWriteCall, TIME_LIMIT_NS};

#[map(name = "SYS_WRITE_MAP")]
pub static SYS_WRITE_MAP: RingBuf = RingBuf::with_byte_size(1024, 0);


#[map(name = "SysWriteIntern")]
static SYS_WRITE_TIMESTAMPS: HashMap<u64, SysWriteIntern> = HashMap::with_max_entries(1024, 0);


struct SysWriteIntern {
    begin_time_stamp: u64,
    fd: i32,
    bytes_written: usize,
}


#[btf_tracepoint]
pub fn sys_enter_write(ctx: BtfTracePointContext) -> Result<(), u32> {
    let id = generate_id(ctx.pid(), ctx.tgid());
    unsafe {
        let data = SysWriteIntern {
            begin_time_stamp: bpf_ktime_get_ns(),
            fd: ctx.arg(0),
            bytes_written: ctx.arg(2),
        };

        match SYS_WRITE_TIMESTAMPS.insert(&id, &data, 0) {
            Ok(_) => Ok(()),
            Err(_) => Err(0),
        }
    }
}


#[btf_tracepoint]
pub fn sys_exit_write(ctx: BtfTracePointContext) -> Result<(), u32> {
    let probe_end = unsafe { bpf_ktime_get_ns() };

    let pid = ctx.pid();
    let tgid = ctx.tgid();
    let call_id = generate_id(pid, tgid);
    let data = match unsafe { SYS_WRITE_TIMESTAMPS.get(&call_id) } {
        None => {return Err(0)}
        Some(entry) => {entry}
    };

    if  probe_end - data.begin_time_stamp > TIME_LIMIT_NS || data.bytes_written == 187 {
        let data = SysWriteCall::new(pid, tgid, data.begin_time_stamp, data.fd, data.bytes_written);

        let mut entry = match SYS_WRITE_MAP.reserve::<SysWriteCall>(0) {
            Some(entry) => entry,
            None => return Err(0),
        };

        entry.write(data);
        entry.submit(0);
    }

    Ok(())
}