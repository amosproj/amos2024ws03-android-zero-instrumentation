// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya::{
    programs::{
        raw_trace_point::RawTracePointLink, trace_point::TracePointLink, RawTracePoint, TracePoint,
    },
    EbpfError,
};
use ractor::ActorRef;
use shared::config::SysSendmsgConfig;

use crate::{
    features::Feature,
    registry::{EbpfRegistry, OwnedHashMap, RegistryGuard},
    symbols::actors::SymbolActorMsg,
};

pub struct SysSendmsgFeature {
    sys_enter_blocking: RegistryGuard<RawTracePoint>,
    sys_exit_blocking: RegistryGuard<RawTracePoint>,
    sys_enter_blocking_link: Option<RawTracePointLink>,
    sys_exit_blocking_link: Option<RawTracePointLink>,
}

impl SysSendmsgFeature {
    fn create(registry: &EbpfRegistry) -> Self {
        Self {
            sys_enter_blocking: registry.program.sys_enter_blocking.take(),
            sys_exit_blocking: registry.program.sys_exit_blocking.take(),
            sys_enter_blocking_link: None,
            sys_exit_blocking_link: None,
        }
    }

    fn attach(&mut self) -> Result<(), EbpfError> {
        if self.sys_enter_blocking_link.is_none() {
            let link_id = self.sys_enter_blocking.attach("sys_enter")?;
            self.sys_enter_blocking_link = Some(self.sys_enter_blocking.take_link(link_id)?);
        }

        if self.sys_exit_blocking_link.is_none() {
            let link_id = self.sys_exit_blocking.attach("sys_exit")?;
            self.sys_exit_blocking_link = Some(self.sys_exit_blocking.take_link(link_id)?);
        }

        Ok(())
    }

    fn detach(&mut self) {
        let _ = self.sys_enter_blocking_link.take();
        let _ = self.sys_exit_blocking_link.take();
    }
}

impl Feature for SysSendmsgFeature {
    type Config = SysSendmsgConfig;
    fn init(registry: &EbpfRegistry, _: Option<ActorRef<SymbolActorMsg>>) -> Self {
        SysSendmsgFeature::create(registry)
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
