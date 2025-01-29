// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::mem::MaybeUninit;

use aya_ebpf::PtRegs;
use ebpf_types::Signal;

use super::SyscallProg;
use crate::{
    event_local::{EventLocal, EventLocalData, EventLocalValue},
    pipeline::{ProgramInfo, SysEnterInfo, SysExitInfo},
    syscalls,
};

impl EventLocalData for Signal {
    type Data = Signal;
}

impl SyscallProg for Signal {
    fn enter<'a>(
        sys_enter: SysEnterInfo,
        _: ProgramInfo,
        mem: &'a mut MaybeUninit<EventLocal<Self>>,
    ) -> Option<&'a mut EventLocal<Self>> {
        initialize_signal_enter(sys_enter.syscall_id, sys_enter.pt_regs, mem)
    }

    fn exit<'a>(
        _: SysExitInfo,
        _: ProgramInfo,
        entry: &EventLocalValue<Self>,
        mem: &'a mut MaybeUninit<Self>,
    ) -> Option<&'a Self> {
        initialize_signal_exit(entry, mem)
    }
}

#[inline(always)]
fn initialize_signal_enter(
    syscall_id: i64,
    pt_regs: PtRegs,
    signal_data: &mut MaybeUninit<EventLocal<Signal>>,
) -> Option<&mut EventLocal<Signal>> {
    if syscall_id != syscalls::SYS_kill {
        return None;
    }

    let ptr = signal_data.as_mut_ptr();
    unsafe {
        (&raw mut (*ptr).data.target_pid).write(pt_regs.arg::<*const u64>(0)? as i32);
        (&raw mut (*ptr).data.signal).write(pt_regs.arg::<*const u64>(1)? as u32);

        Some(signal_data.assume_init_mut())
    }
}

#[inline(always)]
fn initialize_signal_exit<'a>(
    signal_entry: &EventLocalValue<Signal>,
    signal_data: &'a mut MaybeUninit<Signal>,
) -> Option<&'a Signal> {
    let ptr = signal_data.as_mut_ptr();

    unsafe {
        ptr.write(signal_entry.data);

        Some(signal_data.assume_init_mut())
    }
}
