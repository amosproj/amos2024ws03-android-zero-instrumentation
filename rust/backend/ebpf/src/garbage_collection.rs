// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::ptr::read_volatile;

use aya_ebpf::{cty::{c_long, c_void}, helpers::{bpf_get_current_pid_tgid, gen}, macros::{map, uprobe, uretprobe}, maps::{HashMap, RingBuf}, programs::{ProbeContext, RetProbeContext}, EbpfContext};
use aya_log_common::DefaultFormatter;
use aya_log_ebpf::{error, WriteToBuf};
use backend_common::SysGcCall;
use garbage_collection::{read::MemReader, Heap, HeapMetadata};


#[no_mangle]
pub static GC_HEAP_META: Option<HeapMetadata> = None;

#[map]
static GC_HEAP_PTR_INTERNAL: HashMap<u64, usize> = HashMap::pinned(1024, 0);

#[map]
static GC_EVENTS: RingBuf = RingBuf::pinned(size_of::<Heap>() as u32 * 128, 0);

enum Error {
    NoHeapPtr,
    WriteHeapPtr,
    ReadHeapPtr,
    ResetHeapPtr,
    ReadHeap(garbage_collection::read::Error<c_long>),
    RingReserve,
}

impl AsRef<str> for Error {
    fn as_ref(&self) -> &str {
        match self {
            Error::NoHeapPtr => "No heap ptr",
            Error::WriteHeapPtr => "Write heap ptr",
            Error::ReadHeapPtr => "Read heap ptr",
            Error::ResetHeapPtr => "Reset heap ptr",
            Error::ReadHeap(garbage_collection::read::Error::SizeMismatch) => "Heap size mismatch",
            Error::ReadHeap(garbage_collection::read::Error::ReadError(_)) => "Heap read error",
            Error::RingReserve => "Ring reserve error",
        }
    }
}

impl WriteToBuf for Error {
    fn write(self, buf: &mut [u8]) -> Option<core::num::NonZeroUsize> {
        self.as_ref().write(buf)
    }
}

impl DefaultFormatter for Error {}

#[uprobe]
pub fn collect_garbage_internal(ctx: ProbeContext) -> u32 {
    if unsafe { read_volatile(&GC_HEAP_META) }.is_none() {
        error!(&ctx, "No heap metadata");
        return 0;
    }
    
    fn inner(ctx: &ProbeContext) -> Result<(), Error> {
        let arg = ctx.arg::<usize>(0).ok_or(Error::NoHeapPtr)?;
        GC_HEAP_PTR_INTERNAL.insert(&bpf_get_current_pid_tgid(), &arg, 0).map_err(|_| Error::WriteHeapPtr)?;
        Ok(())
    }
    
    if let Err(e) = inner(&ctx) {
        error!(&ctx, "{}", e);
        1
    } else {
        0
    }
}

struct BpfMemReader(usize);

impl MemReader for BpfMemReader {
    type Error = c_long;
    fn read<T>(&self, field: &mut T, offset: usize, len: usize, rel: bool) -> core::result::Result<(), Self::Error> {
        unsafe {
            let len = len.min(size_of::<T>()) as u32;
            let src = if rel {
                (self.0 + offset) as *const c_void
            } else {
                offset as *const c_void
            };


            let ret = gen::bpf_probe_read(
                field as *mut T as *mut c_void,
                len as u32,
                src,
            );
            if ret == 0 {
                Ok(())
            } else {
                Err(ret)
            }
        }
    }
}


#[uretprobe]
pub fn collect_garbage_internal_ret(ctx: RetProbeContext) -> u32 {
    let meta = if let Some(meta) = unsafe { read_volatile(&GC_HEAP_META) } {
        meta
    } else {
        error!(&ctx, "No heap metadata");
        return 0;
    };

    fn inner(ctx: &RetProbeContext, meta: HeapMetadata) -> Result<(), Error> {
        let pid_tgid = bpf_get_current_pid_tgid();
        let pid = ctx.pid();
        let tid = ctx.tgid();
        let arg = *unsafe { GC_HEAP_PTR_INTERNAL.get(&pid_tgid) }.ok_or(Error::ReadHeapPtr)?;
        GC_HEAP_PTR_INTERNAL.remove(&pid_tgid).map_err(|_| Error::ResetHeapPtr)?;
        
        let mut entry = GC_EVENTS.reserve::<SysGcCall>(0).ok_or(Error::RingReserve)?;
        let entry_ptr = entry.as_mut_ptr();
        let heap = unsafe {
            (&raw mut (*entry_ptr).pid).write(pid);
            (&raw mut (*entry_ptr).tid).write(tid);
            (&raw mut (*entry_ptr).heap).write(Heap::default());
            &mut (*entry_ptr).heap
        };
        if let Err(e) = garbage_collection::read::read(heap, meta, &BpfMemReader(arg)).map_err(Error::ReadHeap) {
            entry.discard(0);
            return Err(e);
        }  else {
            entry.submit(0);
        }
        
        Ok(())
    }
    
    if let Err(e) = inner(&ctx, meta) {
        error!(&ctx, "{}", e);
        1
    } else {
        0
    }
}