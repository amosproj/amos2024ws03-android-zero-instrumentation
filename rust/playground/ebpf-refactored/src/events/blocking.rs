// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::mem::MaybeUninit;

use aya_ebpf::helpers::bpf_ktime_get_ns;
use ebpf_types::Blocking;

use super::SyscallProg;
use crate::{
    event_local::{EventLocal, EventLocalData, EventLocalValue},
    pipeline::{ProgramInfo, SysEnterInfo, SysExitInfo},
};

#[repr(C)]
pub struct BlockingEntryData {
    pub syscall_id: u64,
    pub start_time: u64,
}

impl EventLocalData for Blocking {
    type Data = BlockingEntryData;
}

impl SyscallProg for Blocking {
    fn enter<'a>(
        sys_enter: SysEnterInfo,
        _: ProgramInfo,
        mem: &'a mut MaybeUninit<EventLocal<Self>>,
    ) -> Option<&'a mut EventLocal<Self>> {
        initialize_blocking_enter(sys_enter.syscall_id, mem)
    }

    fn exit<'a>(
        _: SysExitInfo,
        _: ProgramInfo,
        entry: &EventLocalValue<Self>,
        mem: &'a mut MaybeUninit<Self>,
    ) -> Option<&'a Self> {
        initialize_blocking_exit(entry, mem)
    }
}

fn initialize_blocking_enter(
    syscall_id: i64,
    blocking_data: &mut MaybeUninit<EventLocal<Blocking>>,
) -> Option<&mut EventLocal<Blocking>> {
    let ptr = blocking_data.as_mut_ptr();

    unsafe {
        (&raw mut (*ptr).data.syscall_id).write(syscall_id as u64);
        (&raw mut (*ptr).data.start_time).write(bpf_ktime_get_ns());

        Some(blocking_data.assume_init_mut())
    }
}

fn initialize_blocking_exit<'a>(
    blocking_entry: &EventLocalValue<Blocking>,
    blocking_data: &'a mut MaybeUninit<Blocking>,
) -> Option<&'a Blocking> {
    let ptr = blocking_data.as_mut_ptr();

    unsafe {
        (&raw mut (*ptr).duration).write(bpf_ktime_get_ns() - blocking_entry.data.start_time);
        (&raw mut (*ptr).syscall_id).write(blocking_entry.data.syscall_id);

        Some(blocking_data.assume_init_mut())
    }
}
