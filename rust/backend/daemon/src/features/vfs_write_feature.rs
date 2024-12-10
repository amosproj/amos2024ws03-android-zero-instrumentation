// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@fau.de>
// SPDX-License-Identifier: MIT

use aya::{Ebpf, EbpfError};
use aya::programs::KProbe;
use aya::programs::kprobe::KProbeLink;
use shared::config::VfsWriteConfig;
use crate::features::{Feature, update_pids};

pub struct VfsWriteFeature {
    vfs_write: Option<KProbeLink>,
    vfs_write_ret: Option<KProbeLink>,
}

impl VfsWriteFeature {

    fn create(ebpf: &mut Ebpf) -> Result<Self, EbpfError> {
        let vfs_write: &mut KProbe = ebpf
            .program_mut("vfs_write")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write".to_string(),
                },
            ))?
            .try_into()?;
        vfs_write.load()?;

        let vfs_write_ret: &mut KProbe = ebpf
            .program_mut("vfs_write_ret")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write_ret".to_string(),
                },
            ))?
            .try_into()?;
        vfs_write_ret.load()?;

        Ok(
            VfsWriteFeature {
                vfs_write: None,
                vfs_write_ret: None
            }
        )
    }

    fn attach(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        if self.vfs_write.is_none() {
            let program: &mut KProbe = ebpf
            .program_mut("vfs_write")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write".to_string(),
                },
            ))?
            .try_into()?;

            let link_id = program.attach("vfs_write",0)?;
            self.vfs_write = Some(program.take_link(link_id)?);
        }

        if self.vfs_write_ret.is_none() {
            let program: &mut KProbe = ebpf
            .program_mut("vfs_write_ret")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write_ret".to_string(),
                },
            ))?
            .try_into()?;
            let link_id = program.attach("vfs_write", 0)?;
            self.vfs_write_ret = Some(program.take_link(link_id)?);

        }

        
        Ok(())
    }

    fn detach(&mut self) {
        let _ = self.vfs_write.take();
        let _ = self.vfs_write_ret.take();
    }

    fn _destroy(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        
        self.detach();
        
        // TODO Error handling
        let vfs_write: &mut KProbe = ebpf
            .program_mut("vfs_write")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write".to_string(),
                },
            ))?
            .try_into()?;
        vfs_write.unload()?;

        let vfs_write_ret: &mut KProbe = ebpf
            .program_mut("vfs_write_ret")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write_ret".to_string(),
                },
            ))?
            .try_into()?;
        vfs_write_ret.unload()?;

        Ok(())
    }


    fn update_pids(
        &self,
        ebpf: &mut Ebpf,
        entries: &std::collections::HashMap<u32, u64>,
    ) -> Result<(), EbpfError> {
        update_pids(ebpf, entries, "VFS_WRITE_PIDS")
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

    fn init(ebpf: &mut Ebpf) -> Self {
        VfsWriteFeature::create(ebpf).expect("Error initializing vfs_write feature")
    }

    fn apply(&mut self, ebpf: &mut Ebpf, config: &Option<Self::Config>) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach(ebpf)?;
                self.update_pids(ebpf, &config.entries)?;
            },
            None => {
                self.detach();
            }
        }
        Ok(())
    }
}
