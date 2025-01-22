// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

#![allow(unused)]
use crate::features::{update_pids, Feature};
use crate::registry::{EbpfRegistry, OwnedHashMap, RegistryGuard};
use crate::symbols::actors::{GetOffsetRequest, SymbolActorMsg};
use aya::programs::trace_point::TracePointLink;
use aya::programs::{TracePoint, UProbe};
use aya::{programs::uprobe::UProbeLink, Ebpf, EbpfError};
use ractor::{call, Actor, ActorRef, RactorErr};
use shared::config::SysFdTrackingConfig;
use tracing_subscriber::{registry, Registry};

pub struct SysFdTrackingFeature {
    trace_create_fd: RegistryGuard<TracePoint>,
    trace_destroy_fd: RegistryGuard<TracePoint>,
    trace_pids: RegistryGuard<OwnedHashMap<u32, u64>>,

    // list of syscalls taken from https://en.wikipedia.org/wiki/File_descriptor
    trace_link_open: Option<TracePointLink>,
    trace_link_creat: Option<TracePointLink>,
    trace_link_socket: Option<TracePointLink>,
    trace_link_accept: Option<TracePointLink>,
    trace_link_socketpair: Option<TracePointLink>,
    trace_link_pipe: Option<TracePointLink>,
    trace_link_epoll_create: Option<TracePointLink>,
    trace_link_signalfd: Option<TracePointLink>,
    trace_link_eventfd: Option<TracePointLink>,
    trace_link_timerfd_create: Option<TracePointLink>,
    trace_link_memfd_create: Option<TracePointLink>,
    trace_link_userfaultfd: Option<TracePointLink>,
    trace_link_inotify_init: Option<TracePointLink>,
    trace_link_clone: Option<TracePointLink>,
    trace_link_pidfd_open: Option<TracePointLink>,

    trace_link_close: Option<TracePointLink>,
    trace_link_close_range: Option<TracePointLink>,
    trace_link_dup: Option<TracePointLink>,

    trace_link_openat: Option<TracePointLink>,
}

impl SysFdTrackingFeature {
    fn create(registry: &EbpfRegistry) -> Self {
        Self {
            trace_create_fd: registry.program.sys_create_fd.take(),
            trace_destroy_fd: registry.program.sys_destroy_fd.take(),
            trace_pids: registry.config.sys_fd_tracking_pids.take(),

            trace_link_open: None,
            trace_link_creat: None,
            trace_link_socket: None,
            trace_link_accept: None,
            trace_link_socketpair: None,
            trace_link_pipe: None,
            trace_link_epoll_create: None,
            trace_link_signalfd: None,
            trace_link_eventfd: None,
            trace_link_timerfd_create: None,
            trace_link_memfd_create: None,
            trace_link_userfaultfd: None,
            trace_link_inotify_init: None,
            trace_link_clone: None,
            trace_link_pidfd_open: None,

            trace_link_close: None,
            trace_link_close_range: None,
            trace_link_dup: None,

            trace_link_openat: None,
        }
    }

    fn try_attach_open(
        trace_create_fd: &mut TracePoint,
        syscall: &str,
    ) -> Result<TracePointLink, EbpfError> {
        let link_id = trace_create_fd.attach("syscalls", syscall)?;
        Ok(trace_create_fd.take_link(link_id)?)
    }
    fn try_attach_destroy(
        trace_destroy_fd: &mut TracePoint,
        syscall: &str,
    ) -> Result<TracePointLink, EbpfError> {
        let link_id = trace_destroy_fd.attach("syscalls", syscall)?;
        Ok(trace_destroy_fd.take_link(link_id)?)
    }

    pub async fn attach(&mut self) -> Result<(), EbpfError> {
        self.trace_link_open
            .get_or_insert(Self::try_attach_open(&mut self.trace_create_fd, "sys_enter_open")?);
        self.trace_link_creat
            .get_or_insert(Self::try_attach_open(&mut self.trace_create_fd, "sys_enter_creat")?);
        self.trace_link_socket
            .get_or_insert(Self::try_attach_open(&mut self.trace_create_fd, "sys_enter_socket")?);
        self.trace_link_accept
            .get_or_insert(Self::try_attach_open(&mut self.trace_create_fd, "sys_enter_accept")?);
        self.trace_link_socketpair
            .get_or_insert(Self::try_attach_open(
                &mut self.trace_create_fd,
                "sys_enter_socketpair",
            )?);
        self.trace_link_pipe
            .get_or_insert(Self::try_attach_open(
                &mut self.trace_create_fd,
                "sys_enter_pipe",
            )?);
        self.trace_link_epoll_create
            .get_or_insert(Self::try_attach_open(
                &mut self.trace_create_fd,
                "sys_enter_epoll_create",
            )?);
        self.trace_link_signalfd
            .get_or_insert(Self::try_attach_open(
                &mut self.trace_create_fd,
                "sys_enter_signalfd",
            )?);
        self.trace_link_eventfd
            .get_or_insert(Self::try_attach_open(&mut self.trace_create_fd, "sys_enter_eventfd")?);
        self.trace_link_timerfd_create
            .get_or_insert(Self::try_attach_open(
                &mut self.trace_create_fd,
                "sys_enter_timerfd_create",
            )?);
        self.trace_link_memfd_create
            .get_or_insert(Self::try_attach_open(
                &mut self.trace_create_fd,
                "sys_enter_memfd_create",
            )?);
        self.trace_link_userfaultfd
            .get_or_insert(Self::try_attach_open(
                &mut self.trace_create_fd,
                "sys_enter_userfaultfd",
            )?);
        self.trace_link_inotify_init
            .get_or_insert(Self::try_attach_open(
                &mut self.trace_create_fd,
                "sys_enter_inotify_init",
            )?);
        self.trace_link_clone
            .get_or_insert(Self::try_attach_open(&mut self.trace_create_fd, "sys_enter_clone")?);
        self.trace_link_pidfd_open
            .get_or_insert(Self::try_attach_open(
                &mut self.trace_create_fd,
                "sys_enter_pidfd_open",
            )?);
        self.trace_link_close
            .get_or_insert(Self::try_attach_destroy(
                &mut self.trace_destroy_fd,
                "sys_enter_close",
            )?);
        self.trace_link_close_range
            .get_or_insert(Self::try_attach_destroy(
                &mut self.trace_destroy_fd,
                "sys_enter_close_range",
            )?);
        self.trace_link_dup.get_or_insert(Self::try_attach_open(
            &mut self.trace_create_fd,
            "sys_enter_dup",
        )?);

        self.trace_link_openat.get_or_insert(Self::try_attach_open(
            &mut self.trace_create_fd,
            "sys_enter_openat",
        )?);

        Ok(())
    }

    pub fn detach(&mut self) {
        let _ = self.trace_link_open.take();
        let _ = self.trace_link_creat.take();
        let _ = self.trace_link_socket.take();
        let _ = self.trace_link_accept.take();
        let _ = self.trace_link_socketpair.take();
        let _ = self.trace_link_pipe.take();
        let _ = self.trace_link_epoll_create.take();
        let _ = self.trace_link_signalfd.take();
        let _ = self.trace_link_eventfd.take();
        let _ = self.trace_link_timerfd_create.take();
        let _ = self.trace_link_memfd_create.take();
        let _ = self.trace_link_userfaultfd.take();
        let _ = self.trace_link_inotify_init.take();
        let _ = self.trace_link_clone.take();
        let _ = self.trace_link_pidfd_open.take();

        let _ = self.trace_link_close.take();
        let _ = self.trace_link_close_range.take();
        let _ = self.trace_link_dup.take();

        let _ = self.trace_link_openat.take();
    }

    fn update_pids(&mut self, pids: &[u32]) -> Result<(), EbpfError> {
        // the general update_pids function for all features works with hashmaps, so the list is converted into a hashmap with keys always being 0
        let pid_0_tuples: Vec<(u32, u64)> = pids.iter().map(|pid| (*pid, 0)).collect();
        let pids_as_hashmap: std::collections::HashMap<u32, u64> =
            std::collections::HashMap::from_iter(pid_0_tuples);

        update_pids(&pids_as_hashmap, &mut self.trace_pids)
    }
}

impl Feature for SysFdTrackingFeature {
    type Config = SysFdTrackingConfig;

    fn init(registry: &EbpfRegistry, _: Option<ActorRef<SymbolActorMsg>>) -> Self {
        SysFdTrackingFeature::create(registry)
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
