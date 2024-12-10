// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@fau.de>
// SPDX-License-Identifier: MIT
#![allow(unused)]
use aya::{
    programs::uprobe::UProbeLink,
    Ebpf, EbpfError,
};
use aya::programs::UProbe;
use crate::features::{update_pids, Feature};
use shared::config::JniReferencesConfig;

pub struct JNIReferencesFeature {
    trace_add_local_link: Option<UProbeLink>,
    trace_del_local_link: Option<UProbeLink>,
    trace_add_global_link: Option<UProbeLink>,
    trace_del_global_link: Option<UProbeLink>,
}

impl JNIReferencesFeature {

    fn create(ebpf: &mut Ebpf) -> Result<Self, EbpfError> {
        Self::jni_load_program_by_name(ebpf, "trace_add_local")?;
        Self::jni_load_program_by_name(ebpf, "trace_del_local")?;
        Self::jni_load_program_by_name(ebpf, "trace_add_global")?;
        Self::jni_load_program_by_name(ebpf, "trace_del_global")?;
        Ok(
            Self {
                trace_add_local_link: None,
                trace_del_local_link: None,
                trace_add_global_link: None,
                trace_del_global_link: None,
            })
    }

    pub fn attach(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        if self.trace_add_local_link.is_none() {
            Self::jni_attach_program_by_name(self, ebpf, "trace_add_local", "AddLocalRef")?
        }

        if self.trace_del_local_link.is_none() {
            Self::jni_attach_program_by_name(self, ebpf, "trace_del_local", "DeleteLocalRef")?
        }

        if self.trace_add_global_link.is_none() {
            Self::jni_attach_program_by_name(self, ebpf, "trace_add_global", "AddGlobalRef")?
        }

        if self.trace_del_global_link.is_none() {
            Self::jni_attach_program_by_name(self, ebpf, "trace_del_global", "DeleteGlobalRef")?
        }

        Ok(())
    }

    pub fn detach(&mut self) {
        let _ = self.trace_add_local_link.take();
        let _ = self.trace_del_local_link.take();
        let _ = self.trace_add_global_link.take();
        let _ = self.trace_del_global_link.take();
    }

    fn destroy(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {

        self.detach();

        // TODO Error handling
        let trace_add_local: &mut UProbe = ebpf
            .program_mut("trace_add_local")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "trace_add_local".to_string(),
                },
            ))?
            .try_into()?;
        trace_add_local.unload()?;

        let trace_del_local: &mut UProbe = ebpf
            .program_mut("trace_del_local")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "trace_del_local".to_string(),
                },
            ))?
            .try_into()?;
        trace_del_local.unload()?;

        let trace_add_global: &mut UProbe = ebpf
            .program_mut("trace_add_global")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "trace_add_global".to_string(),
                },
            ))?
            .try_into()?;
        trace_add_global.unload()?;

        

        let trace_del_global: &mut UProbe = ebpf
            .program_mut("trace_del_global")
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: "trace_del_global".to_string(),
                },
            ))?
            .try_into()?;
        trace_del_global.unload()?;
        
        Ok(())
    }

    fn update_pids(
        &mut self,
        ebpf: &mut Ebpf,
        pids: &[u32]
    ) -> Result<(), EbpfError> {
        
        // the general update_pids function for all features works with hashmaps, so the list is converted into a hashmap with keys always being 0
        let pid_0_tuples: Vec<(u32, u64)> = pids.iter().map(|pid| (*pid, 0)).collect();
        let pids_as_hashmap: std::collections::HashMap<u32, u64> = std::collections::HashMap::from_iter(pid_0_tuples);
        
        update_pids(ebpf,  &pids_as_hashmap, "JNI_REF_PIDS")
    }
    fn jni_load_program_by_name(ebpf: &mut Ebpf, name: &str) -> Result<(), EbpfError> {
        let jni_probe: &mut UProbe = ebpf
            .program_mut(name)
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: name.to_string(),
                },
            ))?
            .try_into()?;
        jni_probe.load()?;
        Ok(())
    }

    fn jni_get_offset_by_name(name: &str) -> u64 {
        todo!("get offset of symbol by name");
    }

    fn jni_get_target_by_name(name: &str) -> &str {
        todo!("get absolute path to library/ binary");
    }

    fn jni_attach_program_by_name(&mut self, ebpf: &mut Ebpf, probe_name: &str, jni_method_name: &str) -> Result<(), EbpfError> {
        let jni_program: &mut UProbe = ebpf
            .program_mut(probe_name)
            .ok_or(EbpfError::ProgramError(
                aya::programs::ProgramError::InvalidName {
                    name: probe_name.to_string(),
                },
            ))?
            .try_into()?;

        let offset = Self::jni_get_offset_by_name(jni_method_name);
        let target = Self::jni_get_target_by_name(jni_method_name);
        let link_id = jni_program.attach(
            Some(jni_method_name),
            offset,
            target,
            None
        ).map_err(|err| {EbpfError::ProgramError(err)})?;
        self.trace_add_local_link = Some(jni_program.take_link(link_id).map_err(|err| {EbpfError::ProgramError(err)})?);
        Ok(())
    }
}

impl Feature for JNIReferencesFeature {
    type Config = JniReferencesConfig;

    fn init(ebpf: &mut Ebpf) -> Self {
        JNIReferencesFeature::create(ebpf).expect("Error initializing JNI reference feature")
    }
    fn apply(
        &mut self,
        ebpf: &mut Ebpf,
        config: &Option<Self::Config>,
    ) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach(ebpf)?;
                self.update_pids(ebpf, &config.pids)?;
            }
            None => {
                self.detach();
            }
        }
        Ok(())
    }
}