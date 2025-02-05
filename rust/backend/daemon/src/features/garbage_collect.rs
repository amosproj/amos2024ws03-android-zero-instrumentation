// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya::{
    programs::{uprobe::UProbeLink, UProbe},
    EbpfError,
};
use ractor::ActorRef;
use shared::config::GarbageCollectConfig;

use crate::{
    features::Feature,
    registry::{EbpfRegistry, RegistryGuard},
    symbols::actors::SymbolActorMsg,
};

// Offset of the collect_garbage_internal function in libart.so
// Found via disassembling and looking at the exported CollectGc method
#[cfg(target_arch = "x86_64")]
const COLLECT_GC_INTERNAL_OFFSET: u64 = 0x57ad10;

// 00000000004aaaf0 <art::gc::Heap::CollectGarbage(bool, art::gc::GcCause)>:
// 4aaaf0: f9419408     	ldr	x8, [x0, #0x328]
// 4aaaf4: 2a0103e3     	mov	w3, w1
// 4aaaf8: 12800004     	mov	w4, #-0x1               // =-1
// 4aaafc: b85fc108     	ldur	w8, [x8, #-0x4]
// 4aab00: 2a0803e1     	mov	w1, w8
// 4aab04: 17fffafb     	b	0x4a96f0 <art::gc::Heap::WaitForGcToComplete(art::gc::GcCause, art::Thread*)+0x990>
#[cfg(target_arch = "aarch64")]
const COLLECT_GC_INTERNAL_OFFSET: u64 = 0x4a96f0;

pub struct GarbageCollectFeature {
    trace_gc_enter: RegistryGuard<UProbe>,
    trace_gc_exit: RegistryGuard<UProbe>,
    trace_enter_gc_link: Option<UProbeLink>,
    trace_exit_gc_link: Option<UProbeLink>,
}

impl GarbageCollectFeature {
    fn create(registry: &EbpfRegistry) -> Self {
        Self {
            trace_gc_enter: registry.program.trace_gc_enter.take(),
            trace_gc_exit: registry.program.trace_gc_exit.take(),
            trace_enter_gc_link: None,
            trace_exit_gc_link: None,
        }
    }

    fn attach(&mut self) -> Result<(), EbpfError> {
        if self.trace_enter_gc_link.is_none() {
            let link_id = self.trace_gc_enter.attach(
                None,
                COLLECT_GC_INTERNAL_OFFSET,
                "/apex/com.android.art/lib64/libart.so",
                None,
            )?;
            self.trace_enter_gc_link = Some(self.trace_gc_enter.take_link(link_id)?);
        }

        if self.trace_exit_gc_link.is_none() {
            let link_id = self.trace_gc_exit.attach(
                None,
                COLLECT_GC_INTERNAL_OFFSET,
                "/apex/com.android.art/lib64/libart.so",
                None,
            )?;
            self.trace_exit_gc_link = Some(self.trace_gc_exit.take_link(link_id)?);
        }

        Ok(())
    }

    fn detach(&mut self) {
        let _ = self.trace_enter_gc_link.take();
        let _ = self.trace_exit_gc_link.take();
    }
}

impl Feature for GarbageCollectFeature {
    type Config = GarbageCollectConfig;

    fn init(registry: &EbpfRegistry, _: Option<ActorRef<SymbolActorMsg>>) -> Self {
        GarbageCollectFeature::create(registry)
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
