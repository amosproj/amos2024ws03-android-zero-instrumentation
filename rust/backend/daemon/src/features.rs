// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@fau.de>
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;
use aya::{
    maps::HashMap,
    programs::{kprobe::KProbeLinkId, uprobe::UProbeLink, trace_point::TracePointLink, KProbe, TracePoint, UProbe, ProgramError},
    Ebpf, EbpfError,
};
use shared::config::Configuration;
use crate::sys_sendmsg_feature::SysSendmsgFeature;
use crate::vfs_write_feature::VfsWriteFeature;



pub trait Feature {
    type Config;

    fn init(ebpf: &mut Ebpf) -> Self;
    fn apply(&mut self, ebpf: &mut Ebpf, config: &Self::Config) -> Result<(), EbpfError>;

}

pub struct Features {
    sys_sendmsg_feature: SysSendmsgFeature,
    vfs_write_feature: VfsWriteFeature,
}

impl Features {

    pub fn init_all_features(ebpf: &mut Ebpf) -> Self {
        let sys_sendmsg_feature = SysSendmsgFeature::init(ebpf).expect("Error when initializing sys_sendmsg feature");
        let vfs_write_feature = VfsWriteFeature::init(ebpf).expect("Error when initializing vfs_write feature");

        Self {
            sys_sendmsg_feature,
            vfs_write_feature,
        }
    }

    pub fn apply_all_features(&mut self, ebpf: &mut Ebpf, config: &mut Configuration) -> Result<(), EbpfError> {
        self.sys_sendmsg_feature.apply(ebpf, &mut config.sys_sendmsg)?;
        self.vfs_write_feature.apply(ebpf, &mut config.vfs_write)?;

        Ok(())
    }

    pub fn update_from_config(
        &mut self,
        ebpf: &mut Ebpf,
        config: &Configuration,
    ) -> Result<(), EbpfError> {


        self.vfs_write_feature.apply(ebpf, &config.vfs_write)?;
        self.sys_sendmsg_feature.apply(ebpf, &config.sys_sendmsg)?;

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

