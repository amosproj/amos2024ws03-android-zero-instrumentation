// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.com>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

#![allow(unused)]
use aya::{
    programs::uprobe::UProbeLink,
    Ebpf, EbpfError,
};
use aya::programs::UProbe;
use tracing_subscriber::{registry, Registry};
use crate::features::{update_pids, Feature};
use crate::registry::{EbpfRegistry, OwnedHashMap, RegistryGuard};
use shared::config::JniReferencesConfig;

pub struct JNIReferencesFeature {
    trace_add_local: RegistryGuard<UProbe>,
    trace_del_local: RegistryGuard<UProbe>,
    trace_add_global: RegistryGuard<UProbe>,
    trace_del_global: RegistryGuard<UProbe>,
    trace_ref_pids: RegistryGuard<OwnedHashMap<u32, u64>>,
    trace_add_local_link: Option<UProbeLink>,
    trace_del_local_link: Option<UProbeLink>,
    trace_add_global_link: Option<UProbeLink>,
    trace_del_global_link: Option<UProbeLink>,
}

impl JNIReferencesFeature {

    fn create(registry: &EbpfRegistry) -> Self {
        Self {
            trace_add_local: registry.program.trace_add_local.take(),
            trace_del_local: registry.program.trace_del_local.take(),
            trace_add_global: registry.program.trace_add_global.take(),
            trace_del_global: registry.program.trace_del_global.take(),
            trace_ref_pids: registry.config.jni_ref_pids.take(),
            trace_add_local_link: None,
            trace_del_local_link: None,
            trace_add_global_link: None,
            trace_del_global_link: None,
        }
    }

    pub fn attach(&mut self) -> Result<(), EbpfError> {
        if self.trace_add_local_link.is_none() {
            self.trace_add_local_link = Some(Self::jni_attach_program_by_name(&mut self.trace_add_local, "trace_add_local", "AddLocalRef")?);
        }

        if self.trace_del_local_link.is_none() {
            self.trace_del_local_link = Some(Self::jni_attach_program_by_name(&mut self.trace_del_local, "trace_del_local", "DeleteLocalRef")?);
        }

        if self.trace_add_global_link.is_none() {
            self.trace_add_global_link = Some(Self::jni_attach_program_by_name( &mut self.trace_add_global, "trace_add_global", "AddGlobalRef")?);
        }

        if self.trace_del_global_link.is_none() {
            self.trace_del_global_link = Some(Self::jni_attach_program_by_name(&mut self.trace_del_global, "trace_del_global", "DeleteGlobalRef")?);
        }

        Ok(())
    }

    pub fn detach(&mut self) {
        let _ = self.trace_add_local_link.take();
        let _ = self.trace_del_local_link.take();
        let _ = self.trace_add_global_link.take();
        let _ = self.trace_del_global_link.take();
    }

    fn update_pids(
        &mut self,
        pids: &[u32]
    ) -> Result<(), EbpfError> {
        
        // the general update_pids function for all features works with hashmaps, so the list is converted into a hashmap with keys always being 0
        let pid_0_tuples: Vec<(u32, u64)> = pids.iter().map(|pid| (*pid, 0)).collect();
        let pids_as_hashmap: std::collections::HashMap<u32, u64> = std::collections::HashMap::from_iter(pid_0_tuples);
        
        update_pids(  &pids_as_hashmap, &mut self.trace_ref_pids)
    }

    fn jni_get_offset_by_name(name: &str) -> u64 {
        todo!("get offset of symbol by name");
    }

    fn jni_get_target_by_name(name: &str) -> &str {
        todo!("get absolute path to library/ binary");
    }

    fn jni_attach_program_by_name(jni_program: &mut RegistryGuard<UProbe>, probe_name: &str, jni_method_name: &str) -> Result<UProbeLink, EbpfError> {
        let offset = Self::jni_get_offset_by_name(jni_method_name);
        let target = Self::jni_get_target_by_name(jni_method_name);
        let link_id = jni_program.attach(
            Some(jni_method_name),
            offset,
            target,
            None
        ).map_err(|err| {EbpfError::ProgramError(err)})?;
        jni_program.take_link(link_id).map_err(|err| {EbpfError::ProgramError(err)})
    }
}

impl Feature for JNIReferencesFeature {
    type Config = JniReferencesConfig;

    fn init(registry: &EbpfRegistry) -> Self {
        JNIReferencesFeature::create(registry)
    }
    fn apply(
        &mut self,
        config: &Option<Self::Config>,
    ) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach()?;
                self.update_pids(&config.pids)?;
            }
            None => {
                self.detach();
            }
        }
        Ok(())
    }
}