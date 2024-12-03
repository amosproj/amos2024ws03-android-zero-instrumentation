// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

#![allow(unused)]

use std::collections::BTreeSet;
use aya::{
    maps::HashMap,
    programs::{kprobe::KProbeLinkId, uprobe::UProbeLinkId, trace_point::TracePointLink, KProbe, TracePoint, UProbe},
    Ebpf, EbpfError,
};
use shared::config::{SysSendmsgConfig, VfsWriteConfig, JniReferencesConfig};


pub struct JNIReferences {
    trace_add_local: Option<UProbeLinkId>,
    trace_del_local: Option<UProbeLinkId>,
    trace_add_global: Option<UProbeLinkId>,
    trace_del_global: Option<UProbeLinkId>,
}

impl JNIReferences {
    pub fn new() -> JNIReferences {
        JNIReferences {
            trace_add_local: None,
            trace_del_local: None,
            trace_add_global: None,
            trace_del_global: None,
        }
    }

    pub fn create(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        // load the 4 programs to hook onto the 4 JNI Reference methods
        let jni_add_local: &mut UProbe = ebpf
            .program_mut("trace_add_local")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "trace_add_local".to_string(),
                },
            ))?
            .try_into()?;
        jni_add_local.load()?;

        let jni_del_local: &mut UProbe = ebpf
            .program_mut("trace_del_local")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "trace_del_local".to_string(),
                },
            ))?
            .try_into()?;
        jni_del_local.load()?;

        let jni_add_global: &mut UProbe = ebpf
            .program_mut("trace_add_global")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "trace_add_global".to_string(),
                },
            ))?
            .try_into()?;
        jni_add_global.load()?;

        let jni_del_global: &mut UProbe = ebpf
            .program_mut("trace_del_global")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "trace_del_global".to_string(),
                },
            ))?
            .try_into()?;
        jni_del_global.load()?;

        Ok(())
    }

    #[allow(unreachable_code)]
    pub fn attach(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        if self.trace_add_local.is_none() {
            let jni_add_local: &mut UProbe = ebpf
                .program_mut("trace_add_local")
                .ok_or(EbpfError::ProgramError(
                    aya::programs::ProgramError::InvalidName {
                        name: "trace_add_local".to_string(),
                    },
                ))?
                .try_into()?;
            let offset = todo!("get offset of symbol");
            let target = todo!("get absolute path to library/ binary");
            /*self.trace_add_local = Some(jni_add_local.attach(
                Some("AddLocalRef"),
                offset,
                target,
                None
            )?);*/
        }

        if self.trace_del_local.is_none() {
            let jni_del_local: &mut UProbe = ebpf
                .program_mut("trace_del_local")
                .ok_or(EbpfError::ProgramError(
                    aya::programs::ProgramError::InvalidName {
                        name: "trace_del_local".to_string(),
                    },
                ))?
                .try_into()?;

            let offset = todo!("get offset of symbol");
            let target = todo!("get absolute path to library/ binary");

            /*self.trace_del_local = Some(jni_del_local.attach(
                Some("DeleteLocalRef"),
                offset,
                target,
                None
            )?);*/
        }

        if self.trace_add_global.is_none() {
            let jni_add_global: &mut UProbe = ebpf
                .program_mut("trace_add_global")
                .ok_or(EbpfError::ProgramError(
                    aya::programs::ProgramError::InvalidName {
                        name: "trace_add_global".to_string(),
                    },
                ))?
                .try_into()?;

            let offset = todo!("get offset of symbol");
            let target = todo!("get absolute path to library/ binary");

            /*self.trace_add_global = Some(jni_add_global.attach(
                Some("AddGlobalRef"),
                offset,
                target,
                None
            )?);*/
        }

        if self.trace_del_global.is_none() {
            let jni_del_global: &mut UProbe = ebpf
                .program_mut("trace_del_global")
                .ok_or(EbpfError::ProgramError(
                    aya::programs::ProgramError::InvalidName {
                        name: "trace_del_global".to_string(),
                    },
                ))?
                .try_into()?;

            let offset = todo!("get offset of symbol");
            let target = todo!("get absolute path to library/ binary");

            /*self.trace_del_global = Some(jni_del_global.attach(
                Some("DeleteGlobalRef"),
                offset,
                target,
                None
            )?);*/
        }

        Ok(())
    }

    pub fn detach(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        // AddLocalRef
        let jni_add_local: &mut UProbe = ebpf
            .program_mut("trace_add_local")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "trace_add_local".to_string(),
                },
            ))?
            .try_into()?;

        if let Some(trace_add_local) = self.trace_add_local.take() {
            jni_add_local.detach(trace_add_local)?;
        }

        //DeleteLocalRef
        let jni_del_local: &mut UProbe = ebpf
            .program_mut("trace_del_local")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "trace_del_local".to_string(),
                },
            ))?
            .try_into()?;

        if let Some(trace_del_local) = self.trace_del_local.take() {
            jni_del_local.detach(trace_del_local)?;
        }

        //AddGlobalRef
        let jni_add_global: &mut UProbe = ebpf
            .program_mut("trace_add_global")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "trace_add_global".to_string(),
                },
            ))?
            .try_into()?;

        if let Some(trace_add_global) = self.trace_add_global.take() {
            jni_add_global.detach(trace_add_global)?;
        }

        //DeleteGlobalRef
        let jni_del_global: &mut UProbe = ebpf
            .program_mut("trace_del_global")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "trace_del_global".to_string(),
                },
            ))?
            .try_into()?;

        if let Some(trace_del_global) = self.trace_del_global.take() {
            jni_del_global.detach(trace_del_global)?;
        }

        Ok(())
    }

    fn update_pids(
        &mut self,
        ebpf: &mut Ebpf,
        pids: &[u32]
    ) -> Result<(), EbpfError> {
        let mut pids_to_track: HashMap<_, u32, u32> = ebpf
            .map_mut("JNI_REF_PIDS")
            .ok_or(EbpfError::MapError(aya::maps::MapError::InvalidName {
                name: "JNI_REF_PIDS".to_string(),
            }))?
            .try_into()?;

        let new_keys = pids.iter().copied().collect::<BTreeSet<u32>>();
        let existing_keys = pids_to_track.keys().collect::<Result<BTreeSet<u32>, _>>()?;

        for key_to_remove in existing_keys.difference(&new_keys) {
            pids_to_track.remove(key_to_remove)?;
        }

        for key_to_add in new_keys.difference(&existing_keys) {
            pids_to_track.insert(key_to_add, 0, 0)?;
        }

        Ok(())
    }

    pub fn apply(
        &mut self,
        ebpf: &mut Ebpf,
        config: Option<&JniReferencesConfig>,
    ) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach(ebpf)?;
                self.update_pids(ebpf, &config.entries)?;
            }
            None => {
                self.detach(ebpf)?;
            }
        }
        Ok(())
    }
}




pub struct SysSendmsgFeature {
    sys_enter_sendmsg_id: Option<TracePointLink>,
    sys_exit_sendmsg_id: Option<TracePointLink>,
}

impl SysSendmsgFeature {
    pub fn new() -> SysSendmsgFeature {
        SysSendmsgFeature {
            sys_enter_sendmsg_id: None,
            sys_exit_sendmsg_id: None,
        }
    }

    fn get_tracepoint_by_name<'a>(
        ebpf: &'a mut Ebpf,
        name: &str,
    ) -> Result<&'a mut TracePoint, EbpfError> {
        Ok(ebpf
            .program_mut(name)
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: name.to_string(),
                },
            ))?
            .try_into()?)
    }

    fn get_sys_enter_sendmsg(ebpf: &mut Ebpf) -> Result<&mut TracePoint, EbpfError> {
        Self::get_tracepoint_by_name(ebpf, "sys_enter_sendmsg")
    }

    fn get_sys_exit_sendmsg(ebpf: &mut Ebpf) -> Result<&mut TracePoint, EbpfError> {
        Self::get_tracepoint_by_name(ebpf, "sys_exit_sendmsg")
    }

    fn detach(&mut self) -> Result<(), EbpfError> {
        let _ = self.sys_enter_sendmsg_id.take();
        let _ = self.sys_exit_sendmsg_id.take();
        Ok(())
    }

    fn attach(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        if self.sys_enter_sendmsg_id.is_none() {
            let p = SysSendmsgFeature::get_sys_enter_sendmsg(ebpf)?;
            let link_id = p.attach("syscalls", "sys_enter_sendmsg")?;
            self.sys_enter_sendmsg_id = Some(p.take_link(link_id)?);
        }

        if self.sys_exit_sendmsg_id.is_none() {
            let p = SysSendmsgFeature::get_sys_exit_sendmsg(ebpf)?;
            let link_id = p.attach("syscalls", "sys_exit_sendmsg")?;
            self.sys_exit_sendmsg_id = Some(p.take_link(link_id)?);
        }

        Ok(())
    }

    fn update_pids(
        &mut self,
        ebpf: &mut Ebpf,
        entries: &std::collections::HashMap<u32, u64>,
    ) -> Result<(), EbpfError> {
        let mut pids_to_track: HashMap<_, u32, u64> = ebpf
            .map_mut("SYS_SENDMSG_PIDS")
            .ok_or(EbpfError::MapError(aya::maps::MapError::InvalidName {
                name: "SYS_SENDMSG_PIDS".to_string(),
            }))?
            .try_into()?;

        let new_keys = entries.keys().copied().collect();
        let existing_keys = pids_to_track.keys().collect::<Result<BTreeSet<u32>, _>>()?;

        for key_to_remove in existing_keys.difference(&new_keys) {
            pids_to_track.remove(key_to_remove)?;
        }

        for (key, value) in entries {
            pids_to_track.insert(key, value, 0)?;
        }

        Ok(())
    }

    pub fn create(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        SysSendmsgFeature::get_sys_exit_sendmsg(ebpf)?.load()?;
        SysSendmsgFeature::get_sys_enter_sendmsg(ebpf)?.load()?;
        Ok(())
    }

    pub fn apply(
        &mut self,
        ebpf: &mut Ebpf,
        config: Option<&SysSendmsgConfig>,
    ) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach(ebpf)?;
                self.update_pids(ebpf, &config.entries)?;
            }
            None => {
                self.detach()?;
            }
        }
        Ok(())
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
        if self.vfs_write_id.is_none() {
            let vfs_write: &mut KProbe = ebpf
                .program_mut("vfs_write")
                .ok_or(EbpfError::ProgramError(
                    aya::programs::ProgramError::InvalidName {
                        name: "vfs_write".to_string(),
                    },
                ))?
                .try_into()?;
            self.vfs_write_id = Some(vfs_write.attach("vfs_write", 0)?);
        }

        if self.vfs_write_ret_id.is_none() {
            let vfs_write_ret: &mut KProbe = ebpf
                .program_mut("vfs_write_ret")
                .ok_or(EbpfError::ProgramError(
                    aya::programs::ProgramError::InvalidName {
                        name: "vfs_write_ret".to_string(),
                    },
                ))?
                .try_into()?;
            self.vfs_write_ret_id = Some(vfs_write_ret.attach("vfs_write", 0)?);
        }

        Ok(())
    }

    pub fn detach(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
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
        }

        Ok(())
    }

    fn update_pids(
        &mut self,
        ebpf: &mut Ebpf,
        entries: &std::collections::HashMap<u32, u64>,
    ) -> Result<(), EbpfError> {
        let mut pids_to_track: HashMap<_, u32, u64> = ebpf
            .map_mut("VFS_WRITE_PIDS")
            .ok_or(EbpfError::MapError(aya::maps::MapError::InvalidName {
                name: "VFS_WRITE_PIDS".to_string(),
            }))?
            .try_into()?;

        let new_keys = entries.keys().copied().collect();
        let existing_keys = pids_to_track.keys().collect::<Result<BTreeSet<u32>, _>>()?;

        for key_to_remove in existing_keys.difference(&new_keys) {
            pids_to_track.remove(key_to_remove)?;
        }

        for (key, value) in entries {
            pids_to_track.insert(key, value, 0)?;
        }

        Ok(())
    }

    pub fn apply(
        &mut self,
        ebpf: &mut Ebpf,
        config: Option<&VfsWriteConfig>,
    ) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach(ebpf)?;
                self.update_pids(ebpf, &config.entries)?;
            }
            None => {
                self.detach(ebpf)?;
            }
        }
        Ok(())
    }
}
