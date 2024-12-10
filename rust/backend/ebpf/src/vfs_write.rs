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
use aya_log_ebpf::error;
use backend_common::{generate_id, VfsWriteCall};


#[map(name = "VFS_WRITE_EVENTS")]
pub static VFS_WRITE_EVENTS: RingBuf = RingBuf::with_byte_size(1024, 0);

#[map(name = "VFS_WRITE_PIDS")]
static VFS_WRITE_PIDS: HashMap<u32, u64> = HashMap::with_max_entries(4096, 0);

#[map(name = "VfsWriteIntern")]
static VFS_WRITE_TIMESTAMPS: HashMap<u64, VfsWriteIntern> = HashMap::with_max_entries(1024, 0);


struct VfsWriteIntern {
    begin_time_stamp: u64,
    fp: u64,
    bytes_written: usize,
}

#[kprobe]
pub fn vfs_write(ctx: ProbeContext) -> Result<(), u32> {
    let pid = ctx.pid();
    let id = generate_id(pid, ctx.tgid());

    if unsafe { VFS_WRITE_PIDS.get(&pid).is_none() } {
        return Ok(());
    }

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

    let duration_threshold_nano_sec = match unsafe { VFS_WRITE_PIDS.get(&pid) } {
        None => return Ok(()), // pid should not be tracked
        Some(duration) => duration,
    };

    let tid = ctx.tgid();
    let call_id = generate_id(pid, tid);
    let data = match unsafe { VFS_WRITE_TIMESTAMPS.get(&call_id) } {
        None => {return Err(0)}
        Some(entry) => {entry}
    };

    let _ = VFS_WRITE_TIMESTAMPS.remove(&call_id);

    if  probe_end - data.begin_time_stamp < *duration_threshold_nano_sec {
        return Ok(());
    }

    let mut entry = match VFS_WRITE_EVENTS.reserve::<VfsWriteCall>(0) {
        Some(entry) => entry,
        None => {
            error!(&ctx, "could not reserve space in map: VFS_WRITE_EVENTS");
            return Err(0)
        },
    };

    let entry_mut = entry.as_mut_ptr();

    unsafe {
        (*entry_mut).pid = pid;
        (*entry_mut).tid = tid;
        (*entry_mut).begin_time_stamp = data.begin_time_stamp;
        (*entry_mut).fp = data.fp;
        (*entry_mut).bytes_written = data.bytes_written;

    }

    entry.submit(0);


    Ok(())
}