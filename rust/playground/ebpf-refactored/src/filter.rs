use ebpf_types::{Equality, EventData, FilterConfig, MissingBehavior, ProcessContext, TaskContext};

use crate::maps::{CMDLINE_FILTER, COMM_FILTER, CONFIG, EXE_PATH_FILTER, PID_FILTER};

fn should_match(eq: Option<&Equality>, mask: u64, missing_behavior: MissingBehavior) -> bool {
    let eq = eq.map(|eq| {
        (
            (eq.eq_for_event_kind & mask) != 0,
            (eq.used_for_event_kind & mask) != 0,
        )
    });
    match (eq, missing_behavior) {
        // The value is not present in the map
        // or not configured for this filter and the missing behavior is to not match
        // or it is configured for this filter but it should not match
        // so we return false
        (None, MissingBehavior::NotMatch)
        | (Some((_, false)), MissingBehavior::NotMatch)
        | (Some((false, _)), _) => false,
        _ => true,
    }
}

/// # Safety
///
/// Ebpf Map operations are unsafe
pub unsafe fn filter<T: EventData>(
    filter_config: &FilterConfig,
    task_context: &TaskContext,
    process_context: &ProcessContext,
) -> bool {
    // do not track ourselves
    if CONFIG.get(0) == Some(&task_context.pid) {
        return true;
    }

    let event_mask = 1u64 << T::EVENT_KIND as u8;

    if let Some(pid_filter) = filter_config.pid_filter {
        let eq = PID_FILTER.get(&task_context.pid);
        if should_match(eq, event_mask, pid_filter.missing_behavior) {
            return false;
        }
        // We take both into account
        let eq = PID_FILTER.get(&task_context.tid);
        if should_match(eq, event_mask, pid_filter.missing_behavior) {
            return false;
        }
    }

    if let Some(comm_filter) = filter_config.comm_filter {
        let eq = COMM_FILTER.get(&task_context.comm);
        if should_match(eq, event_mask, comm_filter.missing_behavior) {
            return false;
        }
    }

    if let Some(exe_path_filter) = filter_config.exe_path_filter {
        let eq = EXE_PATH_FILTER.get(&process_context.exe_path);
        if should_match(eq, event_mask, exe_path_filter.missing_behavior) {
            return false;
        }
    }

    if let Some(cmdline_filter) = filter_config.cmdline_filter {
        let eq = CMDLINE_FILTER.get(&process_context.cmdline);
        if should_match(eq, event_mask, cmdline_filter.missing_behavior) {
            return false;
        }
    }

    filter_config.pid_filter.is_some()
        || filter_config.comm_filter.is_some()
        || filter_config.exe_path_filter.is_some()
        || filter_config.cmdline_filter.is_some()
}
