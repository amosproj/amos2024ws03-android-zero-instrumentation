// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.com>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

#![allow(unused)]
use crate::features::{update_pids, Feature};
use crate::registry::{EbpfRegistry, OwnedHashMap, RegistryGuard};
use crate::symbols::actors::{GetOffsetRequest, SymbolActorMsg};
use aya::programs::UProbe;
use aya::{programs::uprobe::UProbeLink, Ebpf, EbpfError};
use ractor::{call, Actor, ActorRef, RactorErr};
use shared::config::JniReferencesConfig;
use tracing_subscriber::{registry, Registry};

enum JNIMethod {
    AddLocal,
    DelLocal,
    AddGlobal,
    DelGlobal,
}

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
    symbol_actor_ref: ActorRef<SymbolActorMsg>,
}

impl JNIReferencesFeature {
    fn create(registry: &EbpfRegistry, symbol_actor_ref: ActorRef<SymbolActorMsg>) -> Self {
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
            symbol_actor_ref,
        }
    }

    pub async fn attach(&mut self) -> Result<(), EbpfError> {
        if self.trace_add_local_link.is_none() {
            self.trace_add_local_link =
                Some(self.jni_attach_program_by_name(JNIMethod::AddLocal).await?);
        }

        if self.trace_del_local_link.is_none() {
            self.trace_del_local_link =
                Some(self.jni_attach_program_by_name(JNIMethod::DelLocal).await?);
        }

        if self.trace_add_global_link.is_none() {
            self.trace_add_global_link = Some(
                self.jni_attach_program_by_name(JNIMethod::AddGlobal)
                    .await?,
            );
        }

        if self.trace_del_global_link.is_none() {
            self.trace_del_global_link = Some(
                self.jni_attach_program_by_name(JNIMethod::DelGlobal)
                    .await?,
            );
        }

        Ok(())
    }

    pub fn detach(&mut self) {
        let _ = self.trace_add_local_link.take();
        let _ = self.trace_del_local_link.take();
        let _ = self.trace_add_global_link.take();
        let _ = self.trace_del_global_link.take();
    }

    fn update_pids(&mut self, pids: &[u32]) -> Result<(), EbpfError> {
        // the general update_pids function for all features works with hashmaps, so the list is converted into a hashmap with keys always being 0
        let pid_0_tuples: Vec<(u32, u64)> = pids.iter().map(|pid| (*pid, 0)).collect();
        let pids_as_hashmap: std::collections::HashMap<u32, u64> =
            std::collections::HashMap::from_iter(pid_0_tuples);

        update_pids(&pids_as_hashmap, &mut self.trace_ref_pids)
    }

    async fn jni_get_offset_from_name(&self, name: &str) -> Option<u64> {
        call!(
            self.symbol_actor_ref,
            SymbolActorMsg::GetOffset,
            GetOffsetRequest {
                symbol_name: name.to_owned(),
                library_path: "/apex/com.android.art/lib64/libart.so".to_owned(),
            }
        )
        .ok()?
    }

    async fn jni_attach_program_by_name(
        &mut self,
        jni_method: JNIMethod,
    ) -> Result<UProbeLink, EbpfError> {
        let jni_method_name = match (jni_method) {
            JNIMethod::AddLocal => {
                "art::JNIEnvExt::NewLocalRef(art::mirror::Object*)"
            }
            JNIMethod::DelLocal => {
                "art::JNIEnvExt::DeleteLocalRef(_jobject*)"
            }
            JNIMethod::AddGlobal => {
                "art::JavaVMExt::AddGlobalRef(art::Thread*, art::ObjPtr<art::mirror::Object>)"
            }
            JNIMethod::DelGlobal => {
                "art::JavaVMExt::DeleteGlobalRef(art::Thread*, _jobject*)"
            }
        };

        let offset = self.jni_get_offset_from_name(jni_method_name).await;
        if offset.is_none() {
            return Err(EbpfError::BtfError(aya::BtfError::SymbolOffsetNotFound {
                symbol_name: jni_method_name.to_owned(),
            }));
        }

        let jni_program = match (jni_method) {
            JNIMethod::AddLocal => &mut self.trace_add_local,
            JNIMethod::AddGlobal => &mut self.trace_add_global,
            JNIMethod::DelLocal => &mut self.trace_del_local,
            JNIMethod::DelGlobal => &mut self.trace_del_global,
        };

        let link_id = jni_program
            .attach(
                Some(jni_method_name),
                offset.unwrap(),
                "/apex/com.android.art/lib64/libart.so",
                None,
            )
            .map_err(EbpfError::ProgramError)?;
        jni_program
            .take_link(link_id)
            .map_err(EbpfError::ProgramError)
    }
}

impl Feature for JNIReferencesFeature {
    type Config = JniReferencesConfig;

    fn init(registry: &EbpfRegistry, symbol_actor_ref: Option<ActorRef<SymbolActorMsg>>) -> Self {
        JNIReferencesFeature::create(
            registry,
            symbol_actor_ref.expect("Symbol actor must be given."),
        )
    }
    async fn apply(&mut self, config: &Option<Self::Config>) -> Result<(), EbpfError> {
        match config {
            Some(config) => {
                self.attach().await?;
                self.update_pids(&config.pids)?;
            }
            None => {
                self.detach();
            }
        }
        Ok(())
    }
}
