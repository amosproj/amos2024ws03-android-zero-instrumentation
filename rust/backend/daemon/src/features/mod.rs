// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@fau.de>
// SPDX-License-Identifier: MIT

mod jni_reference_feature;
mod vfs_write_feature;
mod sys_sendmsg_feature;

use std::collections::BTreeSet;
use aya::EbpfError;
use jni_reference_feature::JNIReferencesFeature;
use shared::config::Configuration;
use sys_sendmsg_feature::SysSendmsgFeature;
use vfs_write_feature::VfsWriteFeature;

use crate::registry::{EbpfRegistry, OwnedHashMap, RegistryGuard};


pub trait Feature {
    type Config;

    fn init(registry: &EbpfRegistry) -> Self;
    fn apply(&mut self, config: &Option<Self::Config>) -> Result<(), EbpfError>;

}

pub struct Features {
    sys_sendmsg_feature: SysSendmsgFeature,
    vfs_write_feature: VfsWriteFeature,
    jni_reference_feature: JNIReferencesFeature,
}

impl Features {

    pub fn init_all_features(registry: &EbpfRegistry) -> Self {
        let sys_sendmsg_feature = SysSendmsgFeature::init(registry);
        let vfs_write_feature = VfsWriteFeature::init(registry);
        let jni_reference_feature = JNIReferencesFeature::init(registry);

        Self {
            sys_sendmsg_feature,
            vfs_write_feature,
            jni_reference_feature,
        }
    }

    pub fn update_from_config(
        &mut self,
        config: &Configuration,
    ) -> Result<(), EbpfError> {


        self.vfs_write_feature.apply(&config.vfs_write)?;
        self.sys_sendmsg_feature.apply(&config.sys_sendmsg)?;
        self.jni_reference_feature.apply( &config.jni_references)?;

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

