use aya_ebpf::PtRegs;
use ebpf_types::Signal;

use crate::syscalls;

pub fn initialize_signal_enter(
    syscall_id: i64,
    pt_regs: PtRegs,
    signal_data: &mut Signal,
) -> Option<()> {
    if syscall_id != syscalls::SYS_kill {
        return None;
    }

    signal_data.target_pid = pt_regs.arg::<*const u64>(0)? as i32;
    signal_data.signal = pt_regs.arg::<*const u64>(1)? as u32;

    Some(())
}
