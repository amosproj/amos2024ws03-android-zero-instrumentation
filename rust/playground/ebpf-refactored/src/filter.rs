
use aya_ebpf::maps::{Array, HashMap};
use ebpf_types::{
    Equality, EventData, EventKind, FilterConfig, MissingBehavior,
};

pub struct Filter<K: 'static, V: Matcher + 'static>(&'static HashMap<K, V>);
pub struct FilterConfigs {
    config: &'static Array<FilterConfig>,
    host_pid: &'static Array<u32>,
    pid_filter: Filter<u32, Equality>,
    comm_filter: Filter<[u8; 16], Equality>,
    exe_path_filter: Filter<[u8; 4096], Equality>,
    cmdline_filter: Filter<[u8; 256], Equality>,
}

pub trait Matcher {
    fn matches(&self, kind: EventKind) -> Match;
}

impl Matcher for Equality {
    fn matches(&self, kind: EventKind) -> Match {
        let mask = 1u64 << kind as u64;

        if (self.used_for_event_kind & mask) == 0 {
            return Match::NotSpecified;
        }

        if (self.eq_for_event_kind & mask) == 0 {
            Match::Reject
        } else {
            Match::Accept
        }
    }
}

pub enum Match {
    Accept,
    Reject,
    NotSpecified,
}

impl<K: 'static, V: Matcher + 'static> Filter<K, V> {
    pub const fn new(map: &'static HashMap<K, V>) -> Self {
        Self(map)
    }

    pub fn matches(&self, key: &K, kind: EventKind) -> Match {
        match unsafe { self.0.get(key) } {
            Some(matcher) => matcher.matches(kind),
            None => Match::NotSpecified,
        }
    }
}

pub enum FilterEntry<'a> {
    OwnPid(&'a u32),
    Pid(&'a u32),
    Tid(&'a u32),
    Comm(&'a [u8; 16]),
    ExePath(&'a [u8; 4096]),
    Cmdline(&'a [u8; 256]),
}

macro_rules! filter_matches {
    ($filter:expr, $key:expr, $kind:expr, $missing_behavior:expr) => {
        match (
            $filter.matches($key, $kind),
            $missing_behavior.map(|m| m.missing_behavior),
        ) {
            (_, None) => return false,
            (Match::Accept, _) => return false,
            (Match::Reject, _) | (Match::NotSpecified, Some(MissingBehavior::NotMatch)) => {
                return true
            }
            (Match::NotSpecified, Some(MissingBehavior::Match)) => (),
        }
    };
}

impl FilterConfigs {
    pub const fn new(
        config: &'static Array<FilterConfig>,
        host_pid: &'static Array<u32>,
        pid_filter: &'static HashMap<u32, Equality>,
        comm_filter: &'static HashMap<[u8; 16], Equality>,
        exe_path_filter: &'static HashMap<[u8; 4096], Equality>,
        cmdline_filter: &'static HashMap<[u8; 256], Equality>,
    ) -> Self {
        Self {
            config,
            host_pid,
            pid_filter: Filter::new(pid_filter),
            comm_filter: Filter::new(comm_filter),
            exe_path_filter: Filter::new(exe_path_filter),
            cmdline_filter: Filter::new(cmdline_filter),
        }
    }

    fn filter_one_inner<T: EventData>(
        &self,
        entry: &FilterEntry,
        filter_config: &FilterConfig,
    ) -> bool {
        match entry {
            FilterEntry::OwnPid(pid) => {
                if Some(*pid) == self.host_pid.get(0) {
                    return true;
                }
            }
            FilterEntry::Pid(pid) => filter_matches!(
                self.pid_filter,
                pid,
                T::EVENT_KIND,
                filter_config.pid_filter
            ),
            FilterEntry::Tid(tid) => filter_matches!(
                self.pid_filter,
                tid,
                T::EVENT_KIND,
                filter_config.pid_filter
            ),
            FilterEntry::Comm(comm) => filter_matches!(
                self.comm_filter,
                comm,
                T::EVENT_KIND,
                filter_config.comm_filter
            ),
            FilterEntry::ExePath(exe_path) => filter_matches!(
                self.exe_path_filter,
                exe_path,
                T::EVENT_KIND,
                filter_config.exe_path_filter
            ),
            FilterEntry::Cmdline(cmdline) => filter_matches!(
                self.cmdline_filter,
                cmdline,
                T::EVENT_KIND,
                filter_config.cmdline_filter
            ),
        }

        false
    }

    pub fn filter_one<T: EventData>(&self, entry: &FilterEntry) -> bool {
        let Some(filter_config) = self.config.get(T::EVENT_KIND as u32) else {
            return true;
        };

        self.filter_one_inner::<T>(entry, filter_config)
    }

    /// true if the event should be filtered
    /// false if it should be sent to userspace
    pub fn filter_many<T: EventData>(&self, entries: &[FilterEntry]) -> bool {
        // do not track ourselves

        let Some(filter_config) = self.config.get(T::EVENT_KIND as u32) else {
            return true;
        };

        for entry in entries {
            if self.filter_one_inner::<T>(entry, filter_config) {
                return true;
            }
        }

        false
    }
}
