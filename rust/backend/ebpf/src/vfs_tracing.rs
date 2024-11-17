// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT



const TIME_LIMIT_NS: u64 = 100_000_000;

use core::ffi::{c_int, c_size_t};
use aya_ebpf::{
    macros::{kprobe, map},
    maps::{HashMap, RingBuf},
    programs::ProbeContext,
    EbpfContext,
    helpers::gen::bpf_ktime_get_ns,
};
use backend_common::{generate_id, VfsWriteCall};



#[map(name = "VFS_WRITE_MAP")]
pub static VFS_WRITE_MAP: RingBuf = RingBuf::with_byte_size(1024, 0);
#[map(name = "VFS_WRITE_INTERN")]
pub static VFS_WRITE_TIMESTAMPS: HashMap<u64, u64> = HashMap::with_max_entries(1024, 0);


#[kprobe]
pub fn vfs_write(ctx: ProbeContext) -> Result<(), u32> {
    let id = generate_id(ctx.pid(), ctx.tgid());
    let time_stamp = unsafe {bpf_ktime_get_ns()};

    match VFS_WRITE_TIMESTAMPS.insert(&id, &time_stamp, 0) {
        Ok(_) => Ok(()),
        Err(_) => Err(0),
    }

}


#[kprobe]
pub fn vfs_write_ret(ctx: ProbeContext) -> Result<(), u32> {
    let probe_end = unsafe { bpf_ktime_get_ns() };

    let pid = ctx.pid();
    let tgid = ctx.tgid();
    let call_id = generate_id(pid, tgid);
    let probe_start = match unsafe { VFS_WRITE_TIMESTAMPS.get(&call_id) } {
        None => {return Err(0)}
        Some(time_stamp) => {time_stamp.clone()}
    };

    if probe_start - probe_end > TIME_LIMIT_NS {
        let fd: c_int = ctx.arg(0).unwrap();
        let count: c_size_t = ctx.arg(2).unwrap();



        let data = VfsWriteCall::new(pid, tgid, probe_start, fd as i32, count as usize);


        let mut entry = match VFS_WRITE_MAP.reserve::<VfsWriteCall>(0) {
            Some(entry) => entry,
            None => return Err(0),
        };

        entry.write(data);
        entry.submit(0);
    }

    Ok(())
}