// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya::{
    programs::{kprobe::KProbeLinkId, uprobe::UProbeLinkId, KProbe},
    Ebpf, EbpfError,
};
pub enum ProbeID {
    KProbeID(KProbeLinkId),
    UProbeID(UProbeLinkId),
}

struct VfsFeature {
    vfs_write_id: KProbeLinkId,
    vfs_write_ret_id: KProbeLinkId,
}

impl VfsFeature {
    fn create(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        let vfs_write: &mut KProbe = ebpf
            .program_mut("vfs_write")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write".to_string(),
                },
            ))?
            .try_into()?;
        vfs_write.load();
        self.vfs_write_id = vfs_write.attach("vfs_write", 0)?;

        let vfs_write_ret: &mut KProbe = ebpf
            .program_mut("vfs_write_ret")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write_ret".to_string(),
                },
            ))?
            .try_into()?;
        vfs_write_ret.load();
        self.vfs_write_ret_id = vfs_write_ret.attach("vfs_write", 0)?;

        Ok(())
    }

    // fn update(&mut self, ebpf: &mut Ebpf) {
    //     // update pids that are attached
    //     !todo!();
    // }

    fn events(&mut self, ebpf: &mut Ebpf) {
        // return buffered stream of events
        // will be discussed by Felix and Beni
        !todo!()
    }

    fn destroy(ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        // TODO Error handling
        let vfs_write: &mut KProbe = ebpf
            .program_mut("vfs_write")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write".to_string(),
                },
            ))?
            .try_into()?;
        vfs_write.unload();

        let vfs_write_ret: &mut KProbe = ebpf
            .program_mut("vfs_write_ret")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "vfs_write_ret".to_string(),
                },
            ))?
            .try_into()?;
        vfs_write_ret.unload();

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

// pub fn update_from_config(
//     ebpf: &mut Ebpf,
//     config_path: &str,
//     loaded_functions: &mut HashMap<String, ProbeID>,
// ) {
//     let entries = load_from_file(config_path).unwrap().entries;

//     for entry in entries {
//         match (entry.attach, loaded_functions.get(entry.ebpf_name.as_str())) {
//             (true, None) => load_function(
//                 ebpf,
//                 loaded_functions,
//                 entry.uprobe_info,
//                 entry.ebpf_name.as_str(),
//                 entry.hook.as_str(),
//             ),
//             (false, Some(_)) => unload_function(ebpf, loaded_functions, entry.ebpf_name.as_str()),
//             _ => {}
//         }
//     }
// }
