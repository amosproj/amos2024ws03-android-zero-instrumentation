// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

mod garbage_collection_feature;
mod jni_reference_feature;
mod sys_fd_tracking_feature;
mod sys_sendmsg_feature;
mod sys_sigquit_feature;
mod vfs_write_feature;

use std::{
    collections::{HashMap, HashSet},
    process::id,
};

use aya::EbpfError;
use ebpf_types::{Equality, EventKind, Filter, FilterConfig, MissingBehavior};
use garbage_collection_feature::GcFeature;
use jni_reference_feature::JNIReferencesFeature;
use ractor::ActorRef;
use shared::config::Configuration;
use sys_fd_tracking_feature::SysFdTrackingFeature;
use sys_sendmsg_feature::SysSendmsgFeature;
use sys_sigquit_feature::SysSigquitFeature;
use vfs_write_feature::VfsWriteFeature;

use crate::{
    registry::{EbpfRegistry, OwnedHashMap, RegistryGuard},
    symbols::actors::SymbolActorMsg,
};

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
    gc_feature: GcFeature,
    sys_fd_tracking_feature: SysFdTrackingFeature,
    pid_filter: RegistryGuard<OwnedHashMap<u32, Equality>>,
}

const NOT_MATCHING_FILTER: FilterConfig = FilterConfig {
    pid_filter: Some(Filter {
        missing_behavior: MissingBehavior::NotMatch,
    }),
    comm_filter: Some(Filter {
        missing_behavior: MissingBehavior::NotMatch,
    }),
    exe_path_filter: Some(Filter {
        missing_behavior: MissingBehavior::NotMatch,
    }),
    cmdline_filter: Some(Filter {
        missing_behavior: MissingBehavior::NotMatch,
    }),
};

impl Features {
    pub fn init_all_features(
        registry: &EbpfRegistry,
        symbol_actor_ref: ActorRef<SymbolActorMsg>,
    ) -> Self {
        let sys_sendmsg_feature = SysSendmsgFeature::init(registry, None);
        let vfs_write_feature = VfsWriteFeature::init(registry, None);
        let jni_reference_feature = JNIReferencesFeature::init(registry, Some(symbol_actor_ref));
        let sys_sigquit_feature = SysSigquitFeature::init(registry, None);
        let gc_feature = GcFeature::init(registry, None);
        let sys_fd_tracking_feature = SysFdTrackingFeature::init(registry, None);
        let mut filter_config = registry.config.filter_config.take();
        let mut config = registry.config.config.take();
        let mut global_blocking_threshold = registry.config.global_blocking_threshold.take();
        let pid_filter = registry.config.pid_filter.take();

        for i in 0..filter_config.len() {
            let _ = filter_config.set(i, NOT_MATCHING_FILTER, 0);
        }

        let _ = config.set(0, id(), 0);
        let _ = global_blocking_threshold.set(0, 32_000_000, 0);

        Self {
            sys_sendmsg_feature,
            vfs_write_feature,
            jni_reference_feature,
            sys_sigquit_feature,
            gc_feature,
            sys_fd_tracking_feature,
            pid_filter,
        }
    }

    pub async fn update_from_config(&mut self, config: &Configuration) -> Result<(), EbpfError> {
        let mut pids_to_enable = HashMap::<u32, u64>::new();

        if let Some(vfs_write_config) = &config.vfs_write {
            for pid in vfs_write_config.entries.keys() {
                let entry = pids_to_enable.entry(*pid).or_default();
                *entry |= 1 << EventKind::Write as u32;
            }
        }

        if let Some(sys_sendmsg_config) = &config.sys_sendmsg {
            for pid in sys_sendmsg_config.entries.keys() {
                let entry = pids_to_enable.entry(*pid).or_default();
                *entry |= 1 << EventKind::Blocking as u32;
            }
        }

        if let Some(sys_sigquit_config) = &config.sys_sigquit {
            for pid in &sys_sigquit_config.pids {
                let entry = pids_to_enable.entry(*pid).or_default();
                *entry |= 1 << EventKind::Signal as u32;
            }
        }

        if let Some(fd_config) = &config.sys_fd_tracking {
            for pid in &fd_config.pids {
                let entry = pids_to_enable.entry(*pid).or_default();
                *entry |= 1 << EventKind::FileDescriptorChange as u32;
            }
        }

        if let Some(jni_reference_config) = &config.jni_references {
            for pid in &jni_reference_config.pids {
                let entry = pids_to_enable.entry(*pid).or_default();
                *entry |= 1 << EventKind::Jni as u32;
            }
        }

        let eq_config = pids_to_enable
            .into_iter()
            .map(|(pid, eq)| {
                let eq = Equality {
                    // We have no != yet
                    eq_for_event_kind: eq,
                    used_for_event_kind: eq,
                };
                (pid, eq)
            })
            .collect::<HashMap<_, _>>();

        let new_pids = eq_config.keys().copied().collect();
        let existing_pids = self.pid_filter.keys().collect::<Result<HashSet<_>, _>>()?;

        for pids_to_remove in existing_pids.difference(&new_pids) {
            self.pid_filter.remove(pids_to_remove)?;
        }

        for (pid, eq) in eq_config {
            self.pid_filter.insert(pid, eq, 0)?;
        }

        self.vfs_write_feature.apply(&config.vfs_write).await?;
        self.sys_sendmsg_feature.apply(&config.sys_sendmsg).await?;
        self.jni_reference_feature
            .apply(&config.jni_references)
            .await?;
        self.sys_sigquit_feature.apply(&config.sys_sigquit).await?;
        self.gc_feature.apply(&config.gc).await?;
        self.sys_fd_tracking_feature
            .apply(&config.sys_fd_tracking)
            .await?;

        Ok(())
    }
}
