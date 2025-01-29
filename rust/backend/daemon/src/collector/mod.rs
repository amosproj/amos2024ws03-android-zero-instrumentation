// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use aya::maps::ring_buf::RingBufItem;
use bytemuck::checked;
use ebpf_types::{
    Blocking, Event as EbpfEvent, EventKind, FileDescriptorChange, FileDescriptorOp,
    GarbageCollect, Jni, Signal, Write,
};
use shared::ziofa::{
    event::EventType, jni_references_event::JniMethodName, log_event::EventData,
    sys_fd_tracking_event::SysFdAction, Event, GcEvent, JniReferencesEvent, LogEvent,
    SysFdTrackingEvent, SysSendmsgEvent, SysSigquitEvent, VfsWriteEvent,
};

mod aggregator;
mod event_dispatcher;
mod ring_buf;
mod supervisor;
mod time_series;

pub use supervisor::{CollectorSupervisor, CollectorSupervisorArguments};

pub trait IntoEvent {
    fn into_event(self) -> Event;
}

impl IntoEvent for RingBufItem<'_> {
    fn into_event(self) -> Event {
        let kind = checked::from_bytes::<EventKind>(&self[..size_of::<EventKind>()]);
        match *kind {
            EventKind::Write => checked::from_bytes::<EbpfEvent<Write>>(&self).into_event(),
            EventKind::Blocking => checked::from_bytes::<EbpfEvent<Blocking>>(&self).into_event(),
            EventKind::Signal => checked::from_bytes::<EbpfEvent<Signal>>(&self).into_event(),
            EventKind::GarbageCollect => {
                checked::from_bytes::<EbpfEvent<GarbageCollect>>(&self).into_event()
            }
            EventKind::FileDescriptorChange => {
                checked::from_bytes::<EbpfEvent<FileDescriptorChange>>(&self).into_event()
            }
            EventKind::Jni => checked::from_bytes::<EbpfEvent<Jni>>(&self).into_event(),
            _ => todo!(),
        }
    }
}

impl IntoEvent for EbpfEvent<Write> {
    fn into_event(self) -> Event {
        Event {
            event_type: Some(EventType::Log(LogEvent {
                event_data: Some(EventData::VfsWrite(VfsWriteEvent {
                    pid: self.context.task.pid,
                    tid: self.context.task.tid,
                    begin_time_stamp: self.context.timestamp,
                    fp: self.data.file_descriptor,
                    bytes_written: self.data.bytes_written,
                })),
            })),
        }
    }
}

impl IntoEvent for EbpfEvent<Signal> {
    fn into_event(self) -> Event {
        Event {
            event_type: Some(EventType::Log(LogEvent {
                event_data: Some(EventData::SysSigquit(SysSigquitEvent {
                    pid: self.context.task.pid,
                    tid: self.context.task.tid,
                    time_stamp: self.context.timestamp,
                    target_pid: self.data.target_pid as u64, // TODO: negative value
                                                             // TODO: signal kind
                })),
            })),
        }
    }
}

impl IntoEvent for EbpfEvent<GarbageCollect> {
    fn into_event(self) -> Event {
        Event {
            event_type: Some(EventType::Log(LogEvent {
                event_data: Some(EventData::Gc(GcEvent {
                    pid: self.context.task.pid,
                    tid: self.context.task.tid,
                    target_footprint: self.data.target_footprint,
                    num_bytes_allocated: self.data.num_bytes_allocated,
                    gc_cause: self.data.gc_cause,
                    freed_bytes: self.data.freed_bytes as i64,
                    freed_objects: self.data.freed_objects,
                    pause_times: vec![],
                    duration_ns: self.data.duration_ns,
                    freed_los_bytes: self.data.freed_los_bytes as i64,
                    freed_los_objects: self.data.freed_los_objects,
                    gcs_completed: self.data.gcs_completed,
                })),
            })),
        }
    }
}

impl IntoEvent for EbpfEvent<FileDescriptorChange> {
    fn into_event(self) -> Event {
        Event {
            event_type: Some(EventType::Log(LogEvent {
                event_data: Some(EventData::SysFdTracking(SysFdTrackingEvent {
                    pid: self.context.task.pid,
                    tid: self.context.task.tid,
                    time_stamp: self.context.timestamp,
                    fd_action: match self.data.operation {
                        FileDescriptorOp::Open => SysFdAction::Created,
                        FileDescriptorOp::Close => SysFdAction::Destroyed,
                    }
                    .into(),
                })),
            })),
        }
    }
}

impl IntoEvent for EbpfEvent<Jni> {
    fn into_event(self) -> Event {
        Event {
            event_type: Some(EventType::Log(LogEvent {
                event_data: Some(EventData::JniReferences(JniReferencesEvent {
                    pid: self.context.task.pid,
                    tid: self.context.task.tid,
                    begin_time_stamp: self.context.timestamp,
                    jni_method_name: match self.data {
                        Jni::AddLocalRef => JniMethodName::AddLocalRef,
                        Jni::DeleteLocalRef => JniMethodName::DeleteLocalRef,
                        Jni::AddGlobalRef => JniMethodName::AddGlobalRef,
                        Jni::DeleteGlobalRef => JniMethodName::DeleteGlobalRef,
                    }
                    .into(),
                })),
            })),
        }
    }
}

impl IntoEvent for EbpfEvent<Blocking> {
    fn into_event(self) -> Event {
        Event {
            event_type: Some(EventType::Log(LogEvent {
                event_data: Some(EventData::SysSendmsg(SysSendmsgEvent {
                    pid: self.context.task.pid,
                    tid: self.context.task.tid,
                    begin_time_stamp: self.context.timestamp,
                    fd: self.data.syscall_id, // TODO: we have blocking event now
                    duration_nano_sec: self.data.duration,
                })),
            })),
        }
    }
}
