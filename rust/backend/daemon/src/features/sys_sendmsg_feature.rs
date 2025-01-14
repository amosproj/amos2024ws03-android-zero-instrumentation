// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya::EbpfError;
use aya::programs::trace_point::TracePointLink;
use aya::programs::TracePoint;
use shared::config::SysSendmsgConfig;
use crate::features::{update_pids, Feature};
use crate::registry::{EbpfRegistry, OwnedHashMap, RegistryGuard};

pub struct SysSendmsgFeature {
    sys_enter_sendmsg: RegistryGuard<TracePoint>,
    sys_exit_sendmsg: RegistryGuard<TracePoint>,
    sys_sendmsg_pids: RegistryGuard<OwnedHashMap<u32, u64>>,
    sys_enter_sendmsg_link: Option<TracePointLink>,
    sys_exit_sendmsg_link: Option<TracePointLink>,
}

impl SysSendmsgFeature {
    fn create(registry: &EbpfRegistry) -> Self {
        Self {
            sys_enter_sendmsg: registry.program.sys_enter_sendmsg.take(),
            sys_exit_sendmsg: registry.program.sys_exit_sendmsg.take(),
            sys_sendmsg_pids: registry.config.sys_sendmsg_pids.take(),
            sys_enter_sendmsg_link: None,
            sys_exit_sendmsg_link: None,
        }
    }

    fn attach(&mut self) -> Result<(), EbpfError> {
        if self.sys_enter_sendmsg_link.is_none() {
            let link_id = self.sys_enter_sendmsg.attach("syscalls","sys_enter_sendmsg")?;
            self.sys_enter_sendmsg_link = Some(self.sys_enter_sendmsg.take_link(link_id)?);
        }

        if self.sys_exit_sendmsg_link.is_none() {
            let link_id = self.sys_exit_sendmsg.attach("syscalls","sys_exit_sendmsg")?;
            self.sys_exit_sendmsg_link = Some(self.sys_exit_sendmsg.take_link(link_id)?);
        }

        Ok(())
    }

    fn detach(&mut self) {
        // the TrakePointLinks will be automatically detached when the reference is dropped
        let _ = self.sys_enter_sendmsg_link.take();
        let _ = self.sys_exit_sendmsg_link.take();
    }

    fn update_pids(
        &mut self,
        entries: &std::collections::HashMap<u32, u64>,
    ) -> Result<(), EbpfError> {
        update_pids(entries, &mut self.sys_sendmsg_pids)
    }
}

impl Feature for SysSendmsgFeature {
    type Config = SysSendmsgConfig;
    fn init(registry: &EbpfRegistry) -> Self {
        SysSendmsgFeature::create(registry)
    }

    fn apply(&mut self, config: &Option<Self::Config>) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach()?;
                self.update_pids(&config.entries)?;
            }
            None => {
                self.detach();
            }
        }
        Ok(())
    }
}





