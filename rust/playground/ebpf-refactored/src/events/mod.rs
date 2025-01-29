// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::mem::MaybeUninit;

use crate::{
    event_local::{EventLocal, EventLocalData, EventLocalValue},
    pipeline::{ProgramInfo, SysEnterInfo, SysExitInfo},
};

pub mod blocking;
pub mod fdtracking;
pub mod signal;
pub mod write;

pub trait SyscallProg: EventLocalData + Sized {
    fn enter<'a>(
        sys_enter: SysEnterInfo,
        program_info: ProgramInfo,
        mem: &'a mut MaybeUninit<EventLocal<Self>>,
    ) -> Option<&'a mut EventLocal<Self>>;
    fn exit<'a>(
        sys_exit: SysExitInfo,
        program_info: ProgramInfo,
        entry: &EventLocalValue<Self>,
        mem: &'a mut MaybeUninit<Self>,
    ) -> Option<&'a Self>;
}
