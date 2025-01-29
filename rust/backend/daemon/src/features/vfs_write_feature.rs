// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya::{
    programs::{kprobe::KProbeLink, raw_trace_point::RawTracePointLink, KProbe, RawTracePoint},
    EbpfError,
};
use ractor::ActorRef;
use shared::config::VfsWriteConfig;
use tracing::debug;

use crate::{
    features::Feature,
    registry::{EbpfRegistry, OwnedHashMap, RegistryGuard},
    symbols::actors::SymbolActorMsg,
};

pub struct VfsWriteFeature {
    sys_enter_write: RegistryGuard<RawTracePoint>,
    sys_exit_write: RegistryGuard<RawTracePoint>,
    sys_enter_write_link: Option<RawTracePointLink>,
    sys_exit_write_link: Option<RawTracePointLink>,
}

impl VfsWriteFeature {
    pub fn create(registry: &EbpfRegistry) -> Self {
        Self {
            sys_enter_write: registry.program.sys_enter_write.take(),
            sys_exit_write: registry.program.sys_exit_write.take(),
            sys_enter_write_link: None,
            sys_exit_write_link: None,
        }
    }

    fn attach(&mut self) -> Result<(), EbpfError> {
        if self.sys_enter_write_link.is_none() {
            let link_id = self.sys_enter_write.attach("sys_enter")?;
            self.sys_enter_write_link = Some(self.sys_enter_write.take_link(link_id)?);
        }

        if self.sys_exit_write_link.is_none() {
            let link_id = self.sys_exit_write.attach("sys_exit")?;
            self.sys_exit_write_link = Some(self.sys_exit_write.take_link(link_id)?);
        }

        Ok(())
    }

    fn detach(&mut self) {
        let _ = self.sys_enter_write_link.take();
        let _ = self.sys_exit_write_link.take();
    }
}

impl Feature for VfsWriteFeature {
    type Config = VfsWriteConfig;

    fn init(registry: &EbpfRegistry, _: Option<ActorRef<SymbolActorMsg>>) -> Self {
        VfsWriteFeature::create(registry)
    }

    async fn apply(&mut self, config: &Option<Self::Config>) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach()?;
            }
            None => {
                self.detach();
            }
        }
        Ok(())
    }
}
