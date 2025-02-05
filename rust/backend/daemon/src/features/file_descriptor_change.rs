// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

#![allow(unused)]
use aya::{
    programs::{
        raw_trace_point::RawTracePointLink, trace_point::TracePointLink, uprobe::UProbeLink,
        RawTracePoint, TracePoint, UProbe,
    },
    Ebpf, EbpfError,
};
use ractor::{call, Actor, ActorRef, RactorErr};
use shared::config::FileDescriptorChangeConfig;
use tracing_subscriber::{registry, Registry};

use crate::{
    features::Feature,
    registry::{EbpfRegistry, OwnedHashMap, RegistryGuard},
    symbols::actors::{GetOffsetRequest, SymbolActorMsg},
};

pub struct FileDescriptorChangeFeature {
    sys_enter_fdtracking: RegistryGuard<RawTracePoint>,
    sys_exit_fdtracking: RegistryGuard<RawTracePoint>,
    sys_enter_fdtracking_link: Option<RawTracePointLink>,
    sys_exit_fdtracking_link: Option<RawTracePointLink>,
}

impl FileDescriptorChangeFeature {
    fn create(registry: &EbpfRegistry) -> Self {
        Self {
            sys_enter_fdtracking: registry.program.sys_enter_fdtracking.take(),
            sys_exit_fdtracking: registry.program.sys_exit_fdtracking.take(),
            sys_enter_fdtracking_link: None,
            sys_exit_fdtracking_link: None,
        }
    }

    pub async fn attach(&mut self) -> Result<(), EbpfError> {
        if self.sys_enter_fdtracking_link.is_none() {
            let link_id = self.sys_enter_fdtracking.attach("sys_enter")?;
            self.sys_enter_fdtracking_link = Some(self.sys_enter_fdtracking.take_link(link_id)?)
        }

        if self.sys_exit_fdtracking_link.is_none() {
            let link_id = self.sys_exit_fdtracking.attach("sys_exit")?;
            self.sys_exit_fdtracking_link = Some(self.sys_exit_fdtracking.take_link(link_id)?)
        }
        Ok(())
    }

    pub fn detach(&mut self) {
        let _ = self.sys_enter_fdtracking_link.take();
        let _ = self.sys_exit_fdtracking_link.take();
    }
}

impl Feature for FileDescriptorChangeFeature {
    type Config = FileDescriptorChangeConfig;

    fn init(registry: &EbpfRegistry, _: Option<ActorRef<SymbolActorMsg>>) -> Self {
        FileDescriptorChangeFeature::create(registry)
    }

    async fn apply(&mut self, config: &Option<Self::Config>) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach().await?;
            }
            None => {
                self.detach();
            }
        }
        Ok(())
    }
}
