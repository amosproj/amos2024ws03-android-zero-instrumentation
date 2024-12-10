// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@fau.de>
// SPDX-License-Identifier: MIT

use aya::{Ebpf, EbpfError};
use aya::programs::KProbe;
use aya::programs::kprobe::KProbeLinkId;
use shared::config::VfsWriteConfig;
use crate::features::{Feature, update_pids};

pub struct VfsWriteFeature {
    vfs_write_id: Option<KProbeLinkId>,
    vfs_write_ret_id: Option<KProbeLinkId>,
}

impl VfsWriteFeature {

    fn create(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
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

        Ok(())
    }

    fn attach(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        let vfs_write: &mut KProbe = ebpf
            .program_mut("vfs_write")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write".to_string(),
                },
            ))?
            .try_into()?;
        self.vfs_write_id = Some(vfs_write.attach("vfs_write", 0)?);

        let vfs_write_ret: &mut KProbe = ebpf
            .program_mut("vfs_write_ret")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write_ret".to_string(),
                },
            ))?
            .try_into()?;
        self.vfs_write_ret_id = Some(vfs_write_ret.attach("vfs_write", 0)?);
        Ok(())
    }

    fn detach(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        let vfs_write: &mut KProbe = ebpf
            .program_mut("vfs_write")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write".to_string(),
                },
            ))?
            .try_into()?;

        if let Some(vfs_write_id) = self.vfs_write_id.take() {
            vfs_write.detach(vfs_write_id)?;
        } else {
            return Err(EbpfError::ProgramError(
                aya::programs::ProgramError::NotAttached,
            ));
        }

        let vfs_write_ret: &mut KProbe = ebpf
            .program_mut("vfs_write_ret")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write_ret".to_string(),
                },
            ))?
            .try_into()?;

        if let Some(vfs_write_ret_id) = self.vfs_write_ret_id.take() {
            vfs_write_ret.detach(vfs_write_ret_id)?;
        } else {
            return Err(EbpfError::ProgramError(
                aya::programs::ProgramError::NotAttached,
            ));
        }

        Ok(())
    }

    fn _destroy(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        
        self.detach(ebpf)?;
        
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
        let mut this = Self { vfs_write_id: None, vfs_write_ret_id: None };
        this.create(ebpf).expect("Error initializing vfs_write feature");
        this
    }

    fn apply(&mut self, ebpf: &mut Ebpf, config: &Option<Self::Config>) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach(ebpf)?;
                self.update_pids(ebpf, &config.entries)?;
            },
            None => {
                self.detach(ebpf)?;
            }
        }
        Ok(())
    }
}
