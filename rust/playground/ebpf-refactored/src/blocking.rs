use aya_ebpf::helpers::bpf_ktime_get_ns;
use ebpf_types::Blocking;

pub fn initialize_blocking_enter(syscall_id: i64, blocking_data: &mut Blocking) -> Option<()> {
    blocking_data.syscall_id = syscall_id as u64;
    blocking_data.duration = unsafe { bpf_ktime_get_ns() };

    Some(())
}

pub fn initialize_blocking_exit(blocking_data: &mut Blocking) -> Option<()> {
    blocking_data.duration = unsafe { bpf_ktime_get_ns() - blocking_data.duration };

    Some(())
}
