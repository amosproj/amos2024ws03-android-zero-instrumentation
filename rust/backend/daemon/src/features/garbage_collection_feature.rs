// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya::programs::uprobe::UProbeLink;
use aya::EbpfError;
use aya::programs::UProbe;
use ractor::ActorRef;
use shared::config::GcConfig;
use crate::features::Feature;
use crate::registry::{EbpfRegistry, RegistryGuard};
use crate::symbols::actors::SymbolActorMsg;

const COLLECT_GC_INTERNAL_OFFSET: u64 = 0x57ad10;

pub struct GcFeature {
    collect_garbage_internal: RegistryGuard<UProbe>,
    collect_garbage_internal_ret: RegistryGuard<UProbe>,
    collect_garbage_internal_link: Option<UProbeLink>,
    collect_garbage_internal_ret_link: Option<UProbeLink>,
}

impl GcFeature {
    fn create(registry: &EbpfRegistry) -> Self {
        Self {
            collect_garbage_internal: registry.program.collect_garbage_internal.take(),
            collect_garbage_internal_ret: registry.program.collect_garbage_internal_ret.take(),
            collect_garbage_internal_link: None,
            collect_garbage_internal_ret_link: None,
        }
    }

    fn attach(&mut self) -> Result<(), EbpfError> {
        if self.collect_garbage_internal_link.is_none() {
            let link_id = self.collect_garbage_internal.attach(None, COLLECT_GC_INTERNAL_OFFSET, "/apex/com.android.art/lib64/libart.so", None)?;
            self.collect_garbage_internal_link = Some(self.collect_garbage_internal.take_link(link_id)?);
        }

        if self.collect_garbage_internal_ret_link.is_none() {
            let link_id = self.collect_garbage_internal_ret.attach(None, COLLECT_GC_INTERNAL_OFFSET, "/apex/com.android.art/lib64/libart.so", None)?;
            self.collect_garbage_internal_ret_link = Some(self.collect_garbage_internal_ret.take_link(link_id)?);
        }

        Ok(())
    }

    fn detach(&mut self) {
        let _ = self.collect_garbage_internal_link.take();
        let _ = self.collect_garbage_internal_ret_link.take();
    }

}

impl Feature for GcFeature {
    type Config = GcConfig;

    fn init(registry: &EbpfRegistry, _: Option<ActorRef<SymbolActorMsg>>) -> Self {
        GcFeature::create(registry)
    }

    async fn apply(&mut self, config: &Option<Self::Config>) -> Result<(), EbpfError> {
        match config {
            Some(_) => {
                self.attach()?;
            }
            None => {
                self.detach();
            }
        }
        Ok(())
    }
}





