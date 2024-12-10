// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@fau.de>
// SPDX-License-Identifier: MIT

use aya::{Ebpf, EbpfError};
use aya::programs::trace_point::TracePointLink;
use aya::programs::TracePoint;
use shared::config::SysSendmsgConfig;
use crate::features::{update_pids, Feature};

pub struct SysSendmsgFeature {
    sys_enter_sendmsg: Option<TracePointLink>,
    sys_exit_sendmsg: Option<TracePointLink>,
}

impl SysSendmsgFeature {
    fn create(ebpf: &mut Ebpf) -> Result<Self, EbpfError> {
        SysSendmsgFeature::get_sys_enter_sendmsg(ebpf)?.load()?;
        SysSendmsgFeature::get_sys_exit_sendmsg(ebpf)?.load()?;
        Ok(
            Self {
                sys_enter_sendmsg: None,
                sys_exit_sendmsg: None,
            })
    }

    fn attach(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        if self.sys_enter_sendmsg.is_none() {
            let p = SysSendmsgFeature::get_sys_enter_sendmsg(ebpf)?;
            let link_id = p.attach("syscalls","sys_enter_sendmsg")?;
            self.sys_enter_sendmsg = Some(p.take_link(link_id)?);
        }

        if self.sys_exit_sendmsg.is_none() {
            let p = SysSendmsgFeature::get_sys_exit_sendmsg(ebpf)?;
            let link_id = p.attach("syscalls","sys_exit_sendmsg")?;
            self.sys_exit_sendmsg = Some(p.take_link(link_id)?);
        }

        Ok(())
    }

    fn detach(&mut self) {
        // the TrakePointLinks will be automatically detached when the reference is dropped
        let _ = self.sys_enter_sendmsg.take();
        let _ = self.sys_exit_sendmsg.take();
    }

    // make clippy happy
    fn _destroy(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {

        self.detach();

        // TODO Error handling
        let vfs_write: &mut TracePoint = ebpf
            .program_mut("vfs_write")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write".to_string(),
                },
            ))?
            .try_into()?;
        vfs_write.unload()?;

        let vfs_write_ret: &mut TracePoint = ebpf
            .program_mut("vfs_write_ret")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write_ret".to_string(),
                },
            ))?
            .try_into()?;
        vfs_write_ret.unload()?;

        Ok(())
    }

    fn update_pids(
        &self,
        ebpf: &mut Ebpf,
        entries: &std::collections::HashMap<u32, u64>,
    ) -> Result<(), EbpfError> {
        update_pids(ebpf, entries, "SYS_SENDMSG_PIDS")
    }
    
    
    fn get_tracepoint_by_name<'a>(ebpf: &'a mut Ebpf, name: &str) -> Result<&'a mut TracePoint, EbpfError> {
        Ok(ebpf
            .program_mut(name)
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: name.to_string(),
                },
            ))?
            .try_into()?)
    }

    fn get_sys_enter_sendmsg(ebpf: &mut Ebpf) -> Result<&mut TracePoint, EbpfError> {
        Self::get_tracepoint_by_name(ebpf, "sys_enter_sendmsg")
    }

    fn get_sys_exit_sendmsg(ebpf: &mut Ebpf) -> Result<&mut TracePoint, EbpfError> {
        Self::get_tracepoint_by_name(ebpf, "sys_exit_sendmsg")
    }
}

impl Feature for SysSendmsgFeature {
    type Config = SysSendmsgConfig;
    fn init(ebpf: &mut Ebpf) -> Self {
        SysSendmsgFeature::create(ebpf).expect("Error initializing sys_sendmsg feature")
    }

    fn apply(&mut self, ebpf: &mut Ebpf, config: &Option<Self::Config>) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach(ebpf)?;
                self.update_pids(ebpf, &config.entries)?;
            }
            None => {
                self.detach();
            }
        }
        Ok(())
    }
}





