use aya_ebpf::{
    helpers::{bpf_get_current_task, bpf_ktime_get_ns},
    macros::{map, raw_tracepoint},
    maps::RingBuf,
    programs::RawTracePointContext,
    EbpfContext,
};
use aya_log_ebpf::info;
use ebpf_types::{Event, EventContext, EventKind, TaskContext, VfsWrite};

use crate::task_info::task_info_from_task;

#[raw_tracepoint]
fn task_info_test(ctx: RawTracePointContext) -> Option<*mut TaskContext> {
    info!(&ctx, "task_info_test");
    unsafe { task_info_from_task(bpf_get_current_task() as *mut _) }
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
    let task_context_src = task_info_from_task(bpf_get_current_task() as *mut _)?;
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
