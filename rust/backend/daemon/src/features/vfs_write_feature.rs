// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@fau.de>
// SPDX-License-Identifier: MIT

use aya::EbpfError;
use aya::programs::KProbe;
use aya::programs::kprobe::KProbeLink;
use shared::config::VfsWriteConfig;
use crate::features::{Feature, update_pids};
use crate::registry::{EbpfRegistry, OwnedHashMap, RegistryGuard};

pub struct VfsWriteFeature {
    vfs_write: RegistryGuard<KProbe>,
    vfs_write_ret: RegistryGuard<KProbe>,
    vfs_write_pids: RegistryGuard<OwnedHashMap<u32, u64>>,
    vfs_write_link: Option<KProbeLink>,
    vfs_write_ret_link: Option<KProbeLink>,
}

impl VfsWriteFeature {
    
    pub fn create(registry: &EbpfRegistry) -> Self {
        Self {
            vfs_write: registry.program.vfs_write.take(),
            vfs_write_ret: registry.program.vfs_write_ret.take(),
            vfs_write_pids: registry.config.vfs_write_pids.take(),
            vfs_write_link: None,
            vfs_write_ret_link: None,
        }
    }

    fn attach(&mut self) -> Result<(), EbpfError> {
        if self.vfs_write_link.is_none() {
            let link_id = self.vfs_write.attach("vfs_write", 0)?;
            self.vfs_write_link = Some(self.vfs_write.take_link(link_id)?);
        }

        if self.vfs_write_ret_link.is_none() {
            let link_id = self.vfs_write_ret.attach("vfs_write", 0)?;
            self.vfs_write_ret_link = Some(self.vfs_write_ret.take_link(link_id)?);
        }
        
        Ok(())
    }

    fn detach(&mut self) {
        let _ = self.vfs_write_link.take();
        let _ = self.vfs_write_ret_link.take();
    }

    fn update_pids(
        &mut self,
        entries: &std::collections::HashMap<u32, u64>,
    ) -> Result<(), EbpfError> {
        update_pids(entries, &mut self.vfs_write_pids)
    }

    

    // fn update(&mut self, ebpf: &mut Ebpf) {
    //     // update pids that are attached
    //     !todo!();
    // }

    // fn events(&mut self, ebpf: &mut Ebpf) {
    //     // return buffered stream of events
    //     // will be discussed by Felix and Beni
    // }    
}

impl Feature for VfsWriteFeature {
    type Config = VfsWriteConfig;

    fn init(registry: &EbpfRegistry) -> Self {
        VfsWriteFeature::create(registry)
    }

    fn apply(&mut self, config: &Option<Self::Config>) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach()?;
                self.update_pids(&config.entries)?;
            },
            None => {
                self.detach();
            }
        }
        Ok(())
    }
}
