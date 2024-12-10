// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@fau.de>
// SPDX-License-Identifier: MIT

mod jni_reference_feature;
mod vfs_write_feature;
mod sys_sendmsg_feature;

use std::collections::BTreeSet;
use aya::{
    maps::HashMap,
    Ebpf, EbpfError,
};
use jni_reference_feature::JNIReferencesFeature;
use shared::config::Configuration;
use sys_sendmsg_feature::SysSendmsgFeature;
use vfs_write_feature::VfsWriteFeature;


pub trait Feature {
    type Config;

    fn init(ebpf: &mut Ebpf) -> Self;
    fn apply(&mut self, ebpf: &mut Ebpf, config: &Option<Self::Config>) -> Result<(), EbpfError>;

}

pub struct Features {
    sys_sendmsg_feature: SysSendmsgFeature,
    vfs_write_feature: VfsWriteFeature,
    jni_reference_feature: JNIReferencesFeature,
}

impl Features {

    pub fn init_all_features(ebpf: &mut Ebpf) -> Self {
        let sys_sendmsg_feature = SysSendmsgFeature::init(ebpf);
        let vfs_write_feature = VfsWriteFeature::init(ebpf);
        let jni_reference_feature = JNIReferencesFeature::init(ebpf);

        Self {
            sys_sendmsg_feature,
            vfs_write_feature,
            jni_reference_feature,
        }
    }

    pub fn update_from_config(
        &mut self,
        ebpf: &mut Ebpf,
        config: &Configuration,
    ) -> Result<(), EbpfError> {


        self.vfs_write_feature.apply(ebpf, &config.vfs_write)?;
        self.sys_sendmsg_feature.apply(ebpf, &config.sys_sendmsg)?;
        self.jni_reference_feature.apply(ebpf, &config.jni_references)?;

        Ok(())
    }
}



pub fn update_pids(
    ebpf: &mut Ebpf,
    entries: &std::collections::HashMap<u32, u64>,
    map_name: &str
) -> Result<(), EbpfError> {
    let mut pids_to_track: HashMap<_, u32, u64> = ebpf
        .map_mut(map_name)
        .ok_or(EbpfError::MapError(aya::maps::MapError::InvalidName {
            name: map_name.to_string(),
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

