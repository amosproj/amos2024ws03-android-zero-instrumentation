// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{ffi::CStr, sync::LazyLock, time::Duration};

use aya::maps::ring_buf::RingBufItem;
use procfs::boot_time_secs;
use shared::{events::{file_descriptor_change_event, jni_references_event, BlockingEvent, FileDescriptorChangeEvent, GarbageCollectEvent, JniReferencesEvent, SignalEvent}, google::{self, protobuf::Timestamp}};
use bytemuck::checked;
use ebpf_types::{
    Blocking, Event as EbpfEvent, EventKind as EbpfEventKind, FileDescriptorChange, FileDescriptorOp, GarbageCollect, JniReferences, Signal, Write, WriteSource
};

mod aggregator;
mod event_dispatcher;
mod ring_buf;
mod supervisor;
mod time_series;

use shared::events::{event::EventData, log_event::LogEventData, Event, EventContext, LogEvent, WriteEvent};
pub use supervisor::{CollectorSupervisor, CollectorSupervisorArguments};

static BOOT_TIME: LazyLock<u64> = LazyLock::new(|| {
    boot_time_secs().unwrap()
});

fn duration_since_boot_to_timestamp(nanos: u64) -> google::protobuf::Timestamp {

    Timestamp {
        seconds: (*BOOT_TIME + nanos / 1_000_000_000) as i64,
        nanos: (nanos % 1_000_000_000) as i32,
    }
}

pub trait IntoEvent {
    fn into_event(self) -> Event;
}

impl IntoEvent for RingBufItem<'_> {
    fn into_event(self) -> Event {
        let kind = checked::from_bytes::<EbpfEventKind>(&self[..size_of::<EbpfEventKind>()]);
        match *kind {
            EbpfEventKind::Write => checked::from_bytes::<EbpfEvent<Write>>(&self).into_event(),
            EbpfEventKind::Blocking => checked::from_bytes::<EbpfEvent<Blocking>>(&self).into_event(),
            EbpfEventKind::Signal => checked::from_bytes::<EbpfEvent<Signal>>(&self).into_event(),
            EbpfEventKind::GarbageCollect => {
                checked::from_bytes::<EbpfEvent<GarbageCollect>>(&self).into_event()
            }
            EbpfEventKind::FileDescriptorChange => {
                checked::from_bytes::<EbpfEvent<FileDescriptorChange>>(&self).into_event()
            }
            EbpfEventKind::JniReferences => {
                checked::from_bytes::<EbpfEvent<JniReferences>>(&self).into_event()
            }
            _ => todo!(),
        }
    }
}

impl IntoEvent for EbpfEvent<Write> {
    fn into_event(self) -> Event {
        Event {
            event_data: Some(EventData::Log(LogEvent {
                context: Some(EventContext {
                    pid: self.context.task.pid,
                    tid: self.context.task.tid,
                    timestamp: Some(duration_since_boot_to_timestamp(self.context.timestamp)),
                }),
                log_event_data: Some(LogEventData::Write(WriteEvent {
                    bytes_written: self.data.bytes_written,
                    file_descriptor: self.data.file_descriptor,
                    file_path: CStr::from_bytes_until_nul(&self.data.file_path).unwrap().to_string_lossy().to_string(),
                    source: match self.data.source {
                        WriteSource::Write => shared::events::write_event::WriteSource::Write,
                        WriteSource::Write64 => shared::events::write_event::WriteSource::Write64,
                        WriteSource::WriteV => shared::events::write_event::WriteSource::Writev,
                        WriteSource::WriteV2 => shared::events::write_event::WriteSource::Writev2,
                    }.into(),
                })),
            })),
        }
    }
}

impl IntoEvent for EbpfEvent<Signal> {
    fn into_event(self) -> Event {
        Event {
            event_data: Some(EventData::Log(LogEvent {
                context: Some(EventContext {
                    pid: self.context.task.pid,
                    tid: self.context.task.tid,
                    timestamp: Some(duration_since_boot_to_timestamp(self.context.timestamp)),
                }),
                log_event_data: Some(LogEventData::Signal(SignalEvent {
                    target_pid: self.data.target_pid,
                    signal: self.data.signal,
                })),
            })),
        }
    }
}

impl IntoEvent for EbpfEvent<GarbageCollect> {
    fn into_event(self) -> Event {
        Event {
            event_data: Some(EventData::Log(LogEvent {
                context: Some(EventContext {
                    pid: self.context.task.pid,
                    tid: self.context.task.tid,
                    timestamp: Some(duration_since_boot_to_timestamp(self.context.timestamp)),
                }),
                log_event_data: Some(LogEventData::GarbageCollect(GarbageCollectEvent {
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
            }))
        }
    }
}

impl IntoEvent for EbpfEvent<FileDescriptorChange> {
    fn into_event(self) -> Event {
        Event {
            event_data: Some(EventData::Log(LogEvent {
                context: Some(EventContext {
                    pid: self.context.task.pid,
                    tid: self.context.task.tid,
                    timestamp: Some(duration_since_boot_to_timestamp(self.context.timestamp)),
                }),
                log_event_data: Some(LogEventData::FileDescriptorChange(FileDescriptorChangeEvent {
                    open_file_descriptors: self.data.open_fds,
                    operation: match self.data.operation {
                        FileDescriptorOp::Open => file_descriptor_change_event::FileDescriptorOp::Open,
                        FileDescriptorOp::Close => file_descriptor_change_event::FileDescriptorOp::Close,
                    }.into(),
                })),
            }))
        }
    }
}

impl IntoEvent for EbpfEvent<JniReferences> {
    fn into_event(self) -> Event {
        Event {
            event_data: Some(EventData::Log(LogEvent {
                context: Some(EventContext {
                    pid: self.context.task.pid,
                    tid: self.context.task.tid,
                    timestamp: Some(duration_since_boot_to_timestamp(self.context.timestamp)),
                }),
                log_event_data: Some(LogEventData::JniReferences(JniReferencesEvent {
                    method_name: match self.data {
                        JniReferences::AddLocalRef => jni_references_event::JniMethodName::AddLocalRef,
                        JniReferences::DeleteLocalRef => jni_references_event::JniMethodName::DeleteLocalRef,
                        JniReferences::AddGlobalRef => jni_references_event::JniMethodName::AddGlobalRef,
                        JniReferences::DeleteGlobalRef => jni_references_event::JniMethodName::DeleteGlobalRef,
                    }.into(),
                })),
            }))
        }
    }
}

impl IntoEvent for EbpfEvent<Blocking> {
    fn into_event(self) -> Event {
        Event {
            event_data: Some(EventData::Log(LogEvent {
                context: Some(EventContext {
                    pid: self.context.task.pid,
                    tid: self.context.task.tid,
                    timestamp: Some(duration_since_boot_to_timestamp(self.context.timestamp)),
                }),
                log_event_data: Some(LogEventData::Blocking(BlockingEvent {
                    duration: Some(Duration::from_nanos(self.data.duration).into()),
                })),
            }))
        }
    }
}
