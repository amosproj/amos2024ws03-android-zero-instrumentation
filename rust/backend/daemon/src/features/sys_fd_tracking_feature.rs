// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

#![allow(unused)]
use crate::features::{update_pids, Feature};
use crate::registry::{EbpfRegistry, OwnedHashMap, RegistryGuard};
use crate::symbols::actors::{GetOffsetRequest, SymbolActorMsg};
use aya::programs::trace_point::TracePointLink;
use aya::programs::{TracePoint, UProbe};
use aya::{programs::uprobe::UProbeLink, Ebpf, EbpfError};
use ractor::{call, Actor, ActorRef, RactorErr};
use shared::config::SysFdTrackingConfig;
use tracing_subscriber::{registry, Registry};


pub struct SysFdTrackingFeature {
    trace_create_fd: RegistryGuard<TracePoint>,
    trace_destroy_fd: RegistryGuard<TracePoint>,
    trace_pids: RegistryGuard<OwnedHashMap<u32, u64>>,
    trace_link_fd_open: Option<TracePointLink>,
    trace_link_fd_close: Option<TracePointLink>,
}

impl SysFdTrackingFeature {
    fn create(registry: &EbpfRegistry) -> Self {
        Self {
            trace_create_fd: registry.program.sys_create_fd.take(),
            trace_destroy_fd: registry.program.sys_destroy_fd.take(),
            trace_pids: registry.config.sys_fd_tracking_pids.take(),
            trace_link_fd_open: None,
            trace_link_fd_close: None,
        }
    }

    pub async fn attach(&mut self) -> Result<(), EbpfError> {
        // trace all syscalls which create a fd
        if self.trace_link_fd_open.is_none() {
            let link_id = self.trace_create_fd.attach("syscalls","open")?;
            self.trace_link_fd_open = Some(self.trace_create_fd.take_link(link_id)?);
        }

        // trace all syscalls which destroy a fd
        if self.trace_link_fd_close.is_none() {
            let link_id = self.trace_destroy_fd.attach("syscalls","close")?;
            self.trace_link_fd_close = Some(self.trace_destroy_fd.take_link(link_id)?);
        }

        Ok(())
    }

    pub fn detach(&mut self) {
        let _ = self.trace_link_fd_open.take();
        let _ = self.trace_link_fd_close.take();
    }

    fn update_pids(&mut self, pids: &[u32]) -> Result<(), EbpfError> {
        // the general update_pids function for all features works with hashmaps, so the list is converted into a hashmap with keys always being 0
        let pid_0_tuples: Vec<(u32, u64)> = pids.iter().map(|pid| (*pid, 0)).collect();
        let pids_as_hashmap: std::collections::HashMap<u32, u64> =
            std::collections::HashMap::from_iter(pid_0_tuples);

        update_pids(&pids_as_hashmap, &mut self.trace_pids)
    }
}

impl Feature for SysFdTrackingFeature {
    type Config = SysFdTrackingConfig;

    fn init(registry: &EbpfRegistry, _: Option<ActorRef<SymbolActorMsg>>) -> Self {
        SysFdTrackingFeature::create(registry)
    }
    
    async fn apply(&mut self, config: &Option<Self::Config>) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach().await?;
                self.update_pids(&config.pids)?;
            }
            None => {
                self.detach();
            }
        }
        Ok(())
    }
}
