// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya::EbpfError;
use aya::programs::trace_point::TracePointLink;
use aya::programs::TracePoint;
use shared::config::SysSigquitConfig;
use crate::features::{update_pids, Feature};
use crate::registry::{EbpfRegistry, OwnedHashMap, RegistryGuard};

pub struct SysSigquitFeature {
    sys_enter_sigquit: RegistryGuard<TracePoint>,
    sys_enter_sigquit_link: Option<TracePointLink>,
    trace_sigquit_pids: RegistryGuard<OwnedHashMap<u32, u64>>,
}

impl SysSigquitFeature {
    fn create(registry: &EbpfRegistry) -> Self {
        Self {
            sys_enter_sigquit: registry.program.sys_sigquit.take(),
            sys_enter_sigquit_link: None,
            trace_sigquit_pids: registry.config.sys_sigquit_pids.take(),
        }
    }

    fn attach(&mut self) -> Result<(), EbpfError> {
        if self.sys_enter_sigquit_link.is_none() {
            let link_id = self.sys_enter_sigquit.attach("syscalls","sys_enter_sigquit")?;
            self.sys_enter_sigquit_link = Some(self.sys_enter_sigquit.take_link(link_id)?);
        }

        Ok(())
    }

    fn detach(&mut self) {
        // the TrakePointLinks will be automatically detached when the reference is dropped
        let _ = self.sys_enter_sigquit_link.take();
    }

    fn update_pids(
        &mut self,
        pids: &[u32]
    ) -> Result<(), EbpfError> {

        // the general update_pids function for all features works with hashmaps, so the list is converted into a hashmap with keys always being 0
        let pid_0_tuples: Vec<(u32, u64)> = pids.iter().map(|pid| (*pid, 0)).collect();
        let pids_as_hashmap: std::collections::HashMap<u32, u64> = std::collections::HashMap::from_iter(pid_0_tuples);

        update_pids(&pids_as_hashmap, &mut self.trace_sigquit_pids)
    }
}

impl Feature for SysSigquitFeature {
    type Config = SysSigquitConfig;
    fn init(registry: &EbpfRegistry) -> Self {
        SysSigquitFeature::create(registry)
    }

    fn apply(&mut self, config: &Option<Self::Config>) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach()?;
                self.update_pids(&config.pids)?;
            }
            None => {
                self.detach();
            }
        }
        Ok(())
    }
}





