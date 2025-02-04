// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.com>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

mod blocking;
mod file_descriptor_change;
mod garbage_collect;
mod jni_references;
mod signal;
mod write;

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    process::id,
};

use aya::{EbpfError, Pod};
use blocking::BlockingFeature;
use ebpf_types::{Equality, EventKind, Filter, FilterConfig, MissingBehavior};
use file_descriptor_change::FileDescriptorChangeFeature;
use garbage_collect::GarbageCollectFeature;
use jni_references::JniReferencesFeatures;
use ractor::ActorRef;
use shared::config::{Configuration, StringFilter, UInt32Filter};
use signal::SignalFeature;
use write::WriteFeature;

use crate::{
    registry::{EbpfRegistry, OwnedArray, OwnedHashMap, RegistryGuard},
    symbols::actors::SymbolActorMsg,
};

pub trait Feature {
    type Config;

    fn init(registry: &EbpfRegistry, symbol_actor_ref: Option<ActorRef<SymbolActorMsg>>) -> Self;
    async fn apply(&mut self, config: &Option<Self::Config>) -> Result<(), EbpfError>;
}

pub struct Features {
    blocking_feature: BlockingFeature,
    signal_feature: SignalFeature,
    write_feature: WriteFeature,
    jni_reference_feature: JniReferencesFeatures,
    garbage_collect_feature: GarbageCollectFeature,
    file_descriptor_change_feature: FileDescriptorChangeFeature,
    pid_filter: RegistryGuard<OwnedHashMap<u32, Equality>>,
    comm_filter: RegistryGuard<OwnedHashMap<[u8; 16], Equality>>,
    exe_path_filter: RegistryGuard<OwnedHashMap<[u8; 4096], Equality>>,
    cmdline_filter: RegistryGuard<OwnedHashMap<[u8; 256], Equality>>,
    filter_config: RegistryGuard<OwnedArray<FilterConfig>>,
    blocking_threshold: RegistryGuard<OwnedArray<u64>>,
    config: RegistryGuard<OwnedArray<u32>>,
}

macro_rules! extract_filter_from_config {
    ($config:ident, $kind:path) => {
        ($config.as_ref().and_then(|c| c.filter.as_ref()), $kind)
    };
}

macro_rules! extract_filters {
    ($($config:ident: $kind:path),*) => {
        {
            [
                $(
                    extract_filter_from_config!($config, $kind),
                )*
            ]
            .into_iter()
            .filter_map(|(filter, kind)| filter.map(|f| (f, kind)))
        }
    };
}

macro_rules! update_eq_filters {
    ($filter:expr, $kind:expr, $conf:expr => $({$field:ident, $eqs:ident, $transform:expr}),*) => {
        $(
            if let Some(f) = &$filter.$field {
                f.equalities(&mut $eqs, $kind, $transform);
                $conf.$field = Some(Filter {
                    missing_behavior: f.get_missing_behavior(),
                });
            } else {
                $conf.$field = Some(Filter {
                    missing_behavior: MissingBehavior::NotMatch,
                });
            }
        )*
    };
}

macro_rules! apply_features {
    ($self:expr => $($feature:ident: $config:expr),*) => {
        $(
            $self.$feature.apply($config).await?;
        )*
    };
}

impl Features {
    pub fn init_all_features(
        registry: &EbpfRegistry,
        symbol_actor_ref: ActorRef<SymbolActorMsg>,
    ) -> Self {
        let mut this = Self {
            blocking_feature: BlockingFeature::init(registry, None),
            signal_feature: SignalFeature::init(registry, None),
            write_feature: WriteFeature::init(registry, None),
            jni_reference_feature: JniReferencesFeatures::init(registry, Some(symbol_actor_ref)),
            garbage_collect_feature: GarbageCollectFeature::init(registry, None),
            file_descriptor_change_feature: FileDescriptorChangeFeature::init(registry, None),
            pid_filter: registry.config.pid_filter.take(),
            comm_filter: registry.config.comm_filter.take(),
            exe_path_filter: registry.config.exe_path_filter.take(),
            cmdline_filter: registry.config.cmdline_filter.take(),
            filter_config: registry.config.filter_config.take(),
            blocking_threshold: registry.config.global_blocking_threshold.take(),
            config: registry.config.config.take(),
        };

        this.config.set(0, id(), 0).unwrap();
        this.blocking_threshold.set(0, 32_000_000, 0).unwrap();

        this
    }

    pub async fn update_from_config(&mut self, config: &Configuration) -> Result<(), EbpfError> {
        // Destructure the configuration.
        let Configuration {
            write_config,
            blocking_config,
            jni_references_config,
            signal_config,
            file_descriptor_change_config,
            garbage_collect_config,
            ..
        } = config;

        let configs = extract_filters! {
            write_config: EventKind::Write,
            blocking_config: EventKind::Blocking,
            jni_references_config: EventKind::JniReferences,
            signal_config: EventKind::Signal,
            file_descriptor_change_config: EventKind::FileDescriptorChange,
            garbage_collect_config: EventKind::GarbageCollect
        };

        // Create the various equality maps.
        let mut pid_eqs = HashMap::new();
        let mut comm_eqs = HashMap::new();
        let mut exe_path_eqs = HashMap::new();
        let mut cmdline_eqs = HashMap::new();

        let mut filter_config = [FilterConfig::default(); EventKind::MAX as usize];

        // Update all filter maps and configuration fields.
        for (filter, kind) in configs {
            update_eq_filters! {filter, kind, filter_config[kind as usize] =>
                {pid_filter, pid_eqs, |pid| *pid},
                {comm_filter, comm_eqs, |comm| comm.zero_extend()},
                {exe_path_filter, exe_path_eqs, |exe_path| exe_path.zero_extend()},
                {cmdline_filter, cmdline_eqs, |cmdline| cmdline.zero_extend()}
            };
        }

        // Update the blocking threshold if needed.
        if let Some(threshold) = blocking_config.as_ref().and_then(|c| c.threshold) {
            self.blocking_threshold.set(0, threshold, 0)?;
        }

        // Update the filter configuration and all equality maps.
        for (i, config) in filter_config.iter().enumerate() {
            self.filter_config.set(i as u32, *config, 0)?;
        }
        self.pid_filter.update(pid_eqs)?;
        self.comm_filter.update(comm_eqs)?;
        self.exe_path_filter.update(exe_path_eqs)?;
        self.cmdline_filter.update(cmdline_eqs)?;

        // Apply each feature configuration.

        apply_features! {self =>
            write_feature: write_config,
            blocking_feature: blocking_config,
            jni_reference_feature: jni_references_config,
            signal_feature: signal_config,
            garbage_collect_feature: garbage_collect_config,
            file_descriptor_change_feature: file_descriptor_change_config
        }

        Ok(())
    }
}
enum Entry<T> {
    Match(T),
    NotMatch(T),
}

impl<T> Entry<T> {
    fn value(&self) -> &T {
        match self {
            Entry::Match(value) => value,
            Entry::NotMatch(value) => value,
        }
    }
}

trait FilterExt {
    type Values: Hash + Eq + Clone;
    fn matches(&self) -> impl Iterator<Item = Self::Values>;
    fn not_matches(&self) -> impl Iterator<Item = Self::Values>;
    fn missing_behavior_raw(&self) -> i32;

    fn entries(&self) -> impl Iterator<Item = Entry<Self::Values>> {
        self.matches()
            .map(Entry::Match)
            .chain(self.not_matches().map(Entry::NotMatch))
    }

    fn equalities<K, F>(&self, dest: &mut HashMap<K, Equality>, kind: EventKind, key_fn: F)
    where
        K: Eq + Hash,
        F: Fn(&Self::Values) -> K,
    {
        for entry in self.entries() {
            let ent = dest.entry(key_fn(entry.value())).or_insert(Equality {
                eq_for_event_kind: 0,
                used_for_event_kind: 0,
            });

            ent.used_for_event_kind |= 1 << kind as u32;
            match entry {
                Entry::Match(_) => ent.eq_for_event_kind |= 1 << kind as u32,
                Entry::NotMatch(_) => ent.eq_for_event_kind &= 0 << kind as u32,
            }
        }
    }

    fn get_missing_behavior(&self) -> MissingBehavior {
        let behavior = shared::config::MissingBehavior::try_from(self.missing_behavior_raw());
        match behavior {
            Ok(shared::config::MissingBehavior::Match) => MissingBehavior::Match,
            Ok(shared::config::MissingBehavior::NotMatch) => MissingBehavior::NotMatch,
            Ok(shared::config::MissingBehavior::Unspecified) => MissingBehavior::NotMatch,
            Err(_) => MissingBehavior::NotMatch,
        }
    }
}

impl FilterExt for UInt32Filter {
    type Values = u32;

    fn matches(&self) -> impl Iterator<Item = Self::Values> {
        self.r#match.clone().into_iter()
    }

    fn not_matches(&self) -> impl Iterator<Item = Self::Values> {
        self.not_match.clone().into_iter()
    }

    fn missing_behavior_raw(&self) -> i32 {
        self.missing_behavior
    }
}

impl FilterExt for StringFilter {
    type Values = String;

    fn matches(&self) -> impl Iterator<Item = Self::Values> {
        self.r#match.clone().into_iter()
    }

    fn not_matches(&self) -> impl Iterator<Item = Self::Values> {
        self.not_match.clone().into_iter()
    }

    fn missing_behavior_raw(&self) -> i32 {
        self.missing_behavior
    }
}

trait HashMapUpdate<K: Eq + Hash, V> {
    fn update(&mut self, other: HashMap<K, V>) -> Result<(), EbpfError>;
}

impl<K: Eq + Hash + Pod + Debug, V: Pod + Debug> HashMapUpdate<K, V> for OwnedHashMap<K, V> {
    fn update(&mut self, other: HashMap<K, V>) -> Result<(), EbpfError> {
        let new_values = other.keys().map(Cow::Borrowed).collect::<HashSet<_>>();
        let old_values = self
            .keys()
            .map(|key| key.map(|key| Cow::Owned(key)))
            .collect::<Result<HashSet<_>, _>>()?;
        let to_remove = old_values.difference(&new_values);

        for key in to_remove {
            self.remove(key)?;
        }

        for (key, eq) in other {
            self.insert(key, eq, 0)?;
        }

        Ok(())
    }
}

trait ZeroExtend {
    fn zero_extend<const N: usize>(&self) -> [u8; N];
}

impl<T> ZeroExtend for T
where
    T: AsRef<[u8]>,
{
    fn zero_extend<const N: usize>(&self) -> [u8; N] {
        let bytes = self.as_ref();
        let mut result = [0; N];
        let len = bytes.len().min(result.len());
        result[..len].copy_from_slice(&bytes[..len]);
        result
    }
}
