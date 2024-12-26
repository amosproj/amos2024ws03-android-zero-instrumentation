// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya::EbpfError;
use aya::programs::trace_point::TracePointLink;
use aya::programs::TracePoint;
use shared::config::SysSigquitConfig;
use crate::features::{Feature};
use crate::registry::{EbpfRegistry, OwnedHashMap, RegistryGuard};

pub struct SysSigquitFeature {
    sys_enter_sigquit: RegistryGuard<TracePoint>,
    sys_enter_sigquit_link: Option<TracePointLink>,
}

impl SysSigquitFeature {
    fn create(registry: &EbpfRegistry) -> Self {
        Self {
            sys_enter_sigquit: registry.program.sys_enter_sigquit.take(),
            sys_enter_sigquit_link: None,
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
            }
            None => {
                self.detach();
            }
        }
        Ok(())
    }
}





