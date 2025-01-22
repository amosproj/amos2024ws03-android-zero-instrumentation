use aya_ebpf::{
    helpers::bpf_get_current_task, macros::raw_tracepoint, programs::RawTracePointContext,
};
use aya_log_ebpf::info;
use ebpf_types::TaskContext;

use crate::task_info::task_info_from_task;

#[raw_tracepoint]
fn task_info_test(ctx: RawTracePointContext) -> Option<*mut TaskContext> {
    info!(&ctx, "task_info_test");
    unsafe { task_info_from_task(bpf_get_current_task() as *mut _) }
}
