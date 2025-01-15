// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

mod jni_reference_feature;
mod vfs_write_feature;
mod sys_sendmsg_feature;
mod sys_sigquit_feature;

use std::collections::BTreeSet;
use aya::EbpfError;
use jni_reference_feature::JNIReferencesFeature;
use ractor::ActorRef;
use shared::config::Configuration;
use sys_sendmsg_feature::SysSendmsgFeature;
use sys_sigquit_feature::SysSigquitFeature;
use vfs_write_feature::VfsWriteFeature;

use crate::{registry::{EbpfRegistry, OwnedHashMap, RegistryGuard}, symbols::actors::SymbolActorMsg};


pub trait Feature {
    type Config;

    fn init(registry: &EbpfRegistry, symbol_actor_ref: Option<ActorRef<SymbolActorMsg>>) -> Self;
    async fn apply(&mut self, config: &Option<Self::Config>) -> Result<(), EbpfError>;

}

pub struct Features {
    sys_sendmsg_feature: SysSendmsgFeature,
    sys_sigquit_feature: SysSigquitFeature,
    vfs_write_feature: VfsWriteFeature,
    jni_reference_feature: JNIReferencesFeature,
}

impl Features {

    pub fn init_all_features(registry: &EbpfRegistry, symbol_actor_ref: ActorRef<SymbolActorMsg>) -> Self {
        let sys_sendmsg_feature = SysSendmsgFeature::init(registry, None);
        let vfs_write_feature = VfsWriteFeature::init(registry, None);
        let jni_reference_feature = JNIReferencesFeature::init(registry, Some(symbol_actor_ref));
        let sys_sigquit_feature = SysSigquitFeature::init(registry, None);

        Self {
            sys_sendmsg_feature,
            vfs_write_feature,
            jni_reference_feature,
            sys_sigquit_feature,
        }
    }

    pub async fn update_from_config(
        &mut self,
        config: &Configuration,
    ) -> Result<(), EbpfError> {


        self.vfs_write_feature.apply(&config.vfs_write).await?;
        self.sys_sendmsg_feature.apply(&config.sys_sendmsg).await?;
        self.jni_reference_feature.apply( &config.jni_references).await?;
        self.sys_sigquit_feature.apply( &config.sys_sigquit).await?;

        Ok(())
    }
}



pub fn update_pids(
    entries: &std::collections::HashMap<u32, u64>,
    pids_to_track: &mut RegistryGuard<OwnedHashMap<u32, u64>>
) -> Result<(), EbpfError> {
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

