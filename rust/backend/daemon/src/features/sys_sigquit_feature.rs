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
use shared::config::SysSigquitConfig;

use crate::{
    features::Feature,
    registry::{EbpfRegistry, OwnedHashMap, RegistryGuard},
    symbols::actors::SymbolActorMsg,
};

pub struct SysSigquitFeature {
    sys_enter_signal: RegistryGuard<RawTracePoint>,
    sys_exit_signal: RegistryGuard<RawTracePoint>,

    sys_enter_signal_link: Option<RawTracePointLink>,
    sys_exit_signal_link: Option<RawTracePointLink>,
}

impl SysSigquitFeature {
    fn create(registry: &EbpfRegistry) -> Self {
        Self {
            sys_enter_signal: registry.program.sys_enter_signal.take(),
            sys_exit_signal: registry.program.sys_exit_signal.take(),
            sys_enter_signal_link: None,
            sys_exit_signal_link: None,
        }
    }

    fn attach(&mut self) -> Result<(), EbpfError> {
        if self.sys_enter_signal_link.is_none() {
            let link_id = self.sys_enter_signal.attach("sys_enter")?;
            self.sys_enter_signal_link = Some(self.sys_enter_signal.take_link(link_id)?);
        }

        if self.sys_exit_signal_link.is_none() {
            let link_id = self.sys_exit_signal.attach("sys_exit")?;
            self.sys_exit_signal_link = Some(self.sys_exit_signal.take_link(link_id)?);
        }

        Ok(())
    }

    fn detach(&mut self) {
        let _ = self.sys_enter_signal_link.take();
        let _ = self.sys_exit_signal_link.take();
    }
}

impl Feature for SysSigquitFeature {
    type Config = SysSigquitConfig;
    fn init(registry: &EbpfRegistry, _: Option<ActorRef<SymbolActorMsg>>) -> Self {
        SysSigquitFeature::create(registry)
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
