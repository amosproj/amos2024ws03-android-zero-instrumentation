// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::ptr::slice_from_raw_parts_mut;

use aya_ebpf::{
    bindings::task_struct,
    helpers::{bpf_get_current_task, bpf_ktime_get_ns, bpf_probe_read_kernel_buf},
    macros::{map, raw_tracepoint},
    maps::{PerCpuArray, RingBuf},
    programs::RawTracePointContext,
    EbpfContext,
};
use aya_log_ebpf::info;
use ebpf_types::{Event, EventContext, EventKind, TaskContext, VfsWrite};

use crate::{
    path::{PathComponent, PathWalker},
    relocation_helpers::{Path, TaskStruct},
    task_info::task_info_from_task,
};

#[raw_tracepoint]
fn task_info_test(ctx: RawTracePointContext) -> Option<*mut TaskContext> {
    info!(&ctx, "task_info_test");
    unsafe { task_info_from_task(TaskStruct::new(bpf_get_current_task() as *mut _)) }
}

#[map]
static EVENTS: RingBuf = RingBuf::with_byte_size(8192, 0);

#[raw_tracepoint]
fn vfs_write_test(ctx: RawTracePointContext) -> Option<()> {
    info!(&ctx, "vfs_write");
    let mut entry = EVENTS.reserve::<Event>(0)?;
    match unsafe { try_vfs_write(&ctx, entry.as_mut_ptr()) } {
        Some(_) => entry.submit(0),
        None => {
            info!(&ctx, "vfs_write discard");
            entry.discard(0)
        }
    }
    Some(())
}

#[inline(always)]
unsafe fn try_vfs_write(ctx: &RawTracePointContext, entry: *mut Event) -> Option<()> {
    let task_context_src = task_info_from_task(TaskStruct::new(bpf_get_current_task() as *mut _))?;
    let bytes_written = *(ctx.as_ptr().add(16) as *const u64);

    entry.write(Event {
        context: EventContext {
            task: *task_context_src,
            timestamp: bpf_ktime_get_ns(),
        },
        kind: EventKind::VfsWrite(VfsWrite { bytes_written }),
    });

    Some(())
}

#[raw_tracepoint]
fn bin_path_test(ctx: RawTracePointContext) -> Option<()> {
    let mut bin_path = BINARY_PATH.reserve::<[u8; 4096]>(0)?;
    match unsafe { try_bin_path(&ctx, (*bin_path.as_mut_ptr()).as_mut_slice()) } {
        Some(_) => bin_path.submit(0),
        None => {
            info!(&ctx, "bin_path discard");
            bin_path.discard(0)
        }
    }
    Some(())
}

#[map]
static COMPONENT_BUF: PerCpuArray<[u8; 8192]> = PerCpuArray::with_max_entries(1, 0);

const PATH_MAX: usize = 4096;

pub trait EbpfBoundsCheck {
    /// # SAFETY
    ///
    /// Bound must be a power of two
    unsafe fn bounded(self, bound: usize) -> Option<Self>
    where
        Self: Sized;
}

#[cfg(feature = "bounds-check")]
impl EbpfBoundsCheck for usize {
    #[inline(always)]
    unsafe fn bounded(self, bound: usize) -> Option<Self> {
        let this = self & ((bound << 1) - 1);
        if this & bound != 0 {
            return None;
        } else {
            return Some(this & (bound - 1));
        }
    }
}

#[cfg(not(feature = "bounds-check"))]
impl EbpfBoundsCheck for usize {
    #[inline(always)]
    unsafe fn bounded(self, bound: usize) -> Option<Self> {
        Some(self & (bound - 1))
    }
}

#[map]
static BINARY_PATH: RingBuf = RingBuf::with_byte_size(4096 * 128, 0);

enum Error {
    ProbeRead(i64),
    BoundsCheck,
}

impl From<i64> for Error {
    fn from(value: i64) -> Self {
        Error::ProbeRead(value)
    }
}

struct NewTake<I, const N: usize> {
    iter: I,
    index: usize,
}

impl<I, const N: usize> Iterator for NewTake<I, N>
where
    I: Iterator,
{
    type Item = I::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == N {
            None
        } else {
            self.index += 1;
            self.iter.next()
        }
    }
}

trait IteratorExt {
    fn const_take<const N: usize>(self) -> NewTake<Self, N>
    where
        Self: Sized;
}

impl<I> IteratorExt for I
where
    I: Iterator,
{
    fn const_take<const N: usize>(self) -> NewTake<Self, N>
    where
        Self: Sized,
    {
        NewTake {
            iter: self,
            index: 0,
        }
    }
}

struct Offset<'a> {
    max: usize,
    off: usize,
    buf: &'a mut [u8],
}

impl<'a> Offset<'a> {
    pub fn new(buf: &'a mut [u8], max: usize) -> Self {
        Self { max, off: max, buf }
    }

    unsafe fn next_slice(&mut self, len: usize) -> Option<&'a mut [u8]> {
        self.off = (self.off - len).bounded(self.max)?;

        let ptr = self.buf.as_mut_ptr().add(self.off);
        let slice = &mut *slice_from_raw_parts_mut(ptr, len);

        Some(slice)
    }
}

unsafe fn get_path_str(path: Path, buf: &mut [u8; 8192]) -> Result<usize, Error> {
    let mut offset = Offset::new(buf.as_mut_slice(), PATH_MAX);

    let components = PathWalker::new(path)?
        .const_take::<20>()
        .filter_map(|item| match item {
            PathComponent::Name(name) => Some(name),
            PathComponent::Mount => None,
        });

    for name in components {
        let len = name.len().bounded(PATH_MAX).ok_or(Error::BoundsCheck)?;

        let slice = offset.next_slice(len + 1).ok_or(Error::BoundsCheck)?;

        slice[0] = b'/';
        bpf_probe_read_kernel_buf(name as *const u8, &mut slice[1..])?;
    }

    Ok(offset.off)
}

#[inline(never)]
unsafe fn try_bin_path(ctx: &RawTracePointContext, dst: &mut [u8]) -> Option<()> {
    let task_struct = TaskStruct::new(bpf_get_current_task() as *mut task_struct);

    let mm_struct = task_struct.mm().ok()?;
    let exe_file = mm_struct.exe_file().ok()?;
    let f_path = exe_file.f_path();
    let buf = &mut *COMPONENT_BUF.get_ptr_mut(0)?;

    let off = match get_path_str(f_path, buf) {
        Ok(off) => off,
        Err(Error::ProbeRead(i)) => {
            info!(ctx, "failed to get path: {}", i);
            return None;
        }
        _ => return None,
    };

    let dst = dst.as_mut_ptr();
    let src = (*buf).get(off..PATH_MAX)?.as_ptr();
    let count = PATH_MAX - off;

    core::ptr::copy_nonoverlapping(src, dst, count);

    Some(())
}
