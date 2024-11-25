// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya::{
    programs::{kprobe::KProbeLinkId, KProbe},
    Ebpf, EbpfError,
};
use thiserror::Error;
// pub enum ProbeID {
//     KProbeID(KProbeLinkId),
//     UProbeID(UProbeLinkId),
// }

#[derive(Debug, Error)]
pub enum EbpfErrorWrapper {
    #[error(transparent)]
    EbpfError(#[from] EbpfError),
}

impl From<EbpfErrorWrapper> for tonic::Status {
    fn from(err: EbpfErrorWrapper) -> Self {
        Self::from_error(Box::new(err))
    }
}

pub struct VfsFeature {
    vfs_write_id: Option<KProbeLinkId>,
    vfs_write_ret_id: Option<KProbeLinkId>,
}

impl VfsFeature {
    pub fn new() -> VfsFeature {
        VfsFeature {
            vfs_write_id: None,
            vfs_write_ret_id: None,
        }
    }

    pub fn create(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
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

    pub fn attach(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
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

    pub fn _detach(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
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

    // fn update(&mut self, ebpf: &mut Ebpf) {
    //     // update pids that are attached
    //     !todo!();
    // }

    pub fn _events(&mut self, _ebpf: &mut Ebpf) {
        // return buffered stream of events
        // will be discussed by Felix and Beni
    }

    pub fn _destroy(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
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
}

// fn load_function(
//     ebpf: &mut Ebpf,
//     hash_map: &mut HashMap<String, ProbeID>,
//     probe_type: Option<UprobeConfig>,
//     func: &str,
//     hook: &str,
// ) {
//     /* examples:
//      * func: "kprobetcp"
//      * hook: "tcp_connect"
//      */
//     // TODO: Error checking

//     match probe_type {
//         // UPROBE
//         Some(UprobeConfig {
//             offset,
//             target,
//             pid,
//         }) => {
//             // get ebpf program
//             let program: &mut UProbe = ebpf.program_mut(func).unwrap().try_into().unwrap();

//             // load ebpf program
//             program.load().unwrap();

//             // attach ebpf program and insert its ProbeID into the hash map
//             hash_map.insert(
//                 func.to_string(),
//                 ProbeID::UProbeID(program.attach(Some(hook), offset, target, pid).unwrap()),
//             );
//         }
//         // KPROBE/KRETPROBE
//         None => {
//             let program: &mut KProbe = ebpf.program_mut(func).unwrap().try_into().unwrap();
//             program.load().unwrap();
//             hash_map.insert(
//                 func.to_string(),
//                 ProbeID::KProbeID(program.attach(hook, 0).unwrap()),
//             );
//         }
//     }
// }

// fn unload_function(ebpf: &mut Ebpf, hash_map: &mut HashMap<String, ProbeID>, func: &str) {
//     // get ProbeID and remove it from hash map
//     let probe = hash_map.remove(func).unwrap();

//     match probe {
//         ProbeID::UProbeID(_link_id) => {
//             // get ebpf program
//             let program: &mut UProbe = ebpf.program_mut(func).unwrap().try_into().unwrap();

//             // unload ebpf program
//             program.unload().unwrap();
//         }
//         ProbeID::KProbeID(_link_id) => {
//             let program: &mut KProbe = ebpf.program_mut(func).unwrap().try_into().unwrap();
//             program.unload().unwrap();
//         }
//     }
// }

pub struct State {
    vfs_write_feature: VfsFeature,
}

impl State {
    pub fn new() -> State {
        State {
            vfs_write_feature: VfsFeature::new(),
        }
    }
    
    pub fn init(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        self.vfs_write_feature.create(ebpf)?;

        Ok(())
    }

    pub fn update_from_config(&mut self, ebpf: &mut Ebpf, _config_path: &str) -> Result<(), EbpfError> {
        self.vfs_write_feature.attach(ebpf)?;

        Ok(())
    }
}
