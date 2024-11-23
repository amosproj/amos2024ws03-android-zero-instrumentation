// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya_ebpf::{
    macros::{kprobe, map, kretprobe},
    maps::{HashMap, RingBuf},
    programs::{ProbeContext, RetProbeContext},
    EbpfContext,
    helpers::gen::bpf_ktime_get_ns,
};
use backend_common::{generate_id, VfsWriteCall, TIME_LIMIT_NS};



#[map(name = "VFS_WRITE_MAP")]
pub static VFS_WRITE_MAP: RingBuf = RingBuf::with_byte_size(1024, 0);


#[map(name = "VfsWriteIntern")]
static VFS_WRITE_TIMESTAMPS: HashMap<u64, VfsWriteIntern> = HashMap::with_max_entries(1024, 0);


struct VfsWriteIntern {
    begin_time_stamp: u64,
    fp: u64,
    bytes_written: usize,
}

#[kprobe]
pub fn vfs_write(ctx: ProbeContext) -> Result<(), u32> {
    let id = generate_id(ctx.pid(), ctx.tgid());

    let begin_time_stamp: u64;
    let fp: u64;
    let bytes_written: usize;
    unsafe {
        begin_time_stamp = bpf_ktime_get_ns();
        fp = match ctx.arg(0) {
            Some(arg) => arg,
            None => return Err(0),
        };
        bytes_written = match ctx.arg(2) {
            Some(arg) => arg,
            None => return Err(0),
        };
    }


    let data = VfsWriteIntern { begin_time_stamp, fp, bytes_written };

    match VFS_WRITE_TIMESTAMPS.insert(&id, &data, 0) {
        Ok(_) => Ok(()),
        Err(_) => Err(0),
    }

}


#[kretprobe]
pub fn vfs_write_ret(ctx: RetProbeContext) -> Result<(), u32> {
    let probe_end = unsafe { bpf_ktime_get_ns() };

    let pid = ctx.pid();
    let tgid = ctx.tgid();
    let call_id = generate_id(pid, tgid);
    let data = match unsafe { VFS_WRITE_TIMESTAMPS.get(&call_id) } {
        None => {return Err(0)}
        Some(entry) => {entry}
    };

    if  probe_end - data.begin_time_stamp > TIME_LIMIT_NS {
        let data = VfsWriteCall::new(pid, tgid, data.begin_time_stamp, data.fp, data.bytes_written);

        let mut entry = match VFS_WRITE_MAP.reserve::<VfsWriteCall>(0) {
            Some(entry) => entry,
            None => return Err(0),
        };

        entry.write(data);
        entry.submit(0);
    }

    Ok(())
}