// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use backend_common::{JNICall, JNIMethodName, SysSendmsgCall, SysSigquitCall, VfsWriteCall, SysGcCall};
use shared::ziofa::event::EventType;
use shared::ziofa::{Event, GcEvent, JniReferencesEvent, SysSendmsgEvent, SysSigquitEvent, VfsWriteEvent, LogEvent};
use shared::ziofa::jni_references_event::JniMethodName;
use shared::ziofa::log_event::EventData;

mod aggregator;
mod event_dispatcher;
mod ring_buf;
mod supervisor;
mod time_series;

pub use supervisor::{CollectorSupervisor, CollectorSupervisorArguments};

pub trait IntoEvent {
    fn into_event(self) -> Event;
}

impl IntoEvent for VfsWriteCall {
    fn into_event(self) -> Event {
        Event {
            event_type: Some(EventType::Log(LogEvent {
                event_data: Some(EventData::VfsWrite(VfsWriteEvent {
                    pid: self.pid,
                    tid: self.tid,
                    begin_time_stamp: self.begin_time_stamp,
                    fp: self.fp,
                    bytes_written: self.bytes_written as u64,
                })),
            })),
        }
    }
}

impl IntoEvent for SysSendmsgCall {
    fn into_event(self) -> Event {
        Event {
            event_type: Some(EventType::Log(LogEvent {
                event_data: Some(EventData::SysSendmsg(SysSendmsgEvent {
                    pid: self.pid,
                    tid: self.tid,
                    begin_time_stamp: self.begin_time_stamp,
                    fd: self.fd,
                    duration_nano_sec: self.duration_nano_sec,
                })),
            })),
        }
    }
}

impl IntoEvent for JNICall {
    fn into_event(self) -> Event {
        Event {
            event_type: Some(EventType::Log(LogEvent {
                event_data: Some(EventData::JniReferences(JniReferencesEvent {
                    pid: self.pid,
                    tid: self.tid,
                    begin_time_stamp: self.begin_time_stamp,
                    jni_method_name: (match self.method_name {
                        JNIMethodName::AddLocalRef => JniMethodName::AddLocalRef,
                        JNIMethodName::DeleteLocalRef => JniMethodName::DeleteLocalRef,
                        JNIMethodName::AddGlobalRef => JniMethodName::AddGlobalRef,
                        JNIMethodName::DeleteGlobalRef => JniMethodName::DeleteGlobalRef,
                    })
                    .into(),
                })),
            })),
        }
    }
}

impl IntoEvent for SysSigquitCall {
    fn into_event(self) -> Event {
        Event {
            event_type: Some(EventType::Log(LogEvent {
                event_data: Some(EventData::SysSigquit(SysSigquitEvent {
                    pid: self.pid,
                    tid: self.tid,
                    time_stamp: self.time_stamp,
                    target_pid: self.target_pid,
                }))
            }))
        }
    }
}

impl IntoEvent for SysGcCall {
    fn into_event(self) -> Event {
        Event {
            event_type: Some(EventType::Log(LogEvent {
                event_data: Some(EventData::Gc(GcEvent {
                    pid: self.pid,
                    tid: self.tid,
                    target_footprint: self.heap.target_footprint as u64,
                    num_bytes_allocated: self.heap.num_bytes_allocated as u64,
                    gcs_completed: self.heap.gcs_completed,
                    gc_cause: self.heap.gc_cause as u32,
                    duration_ns: self.heap.duration_ns,
                    freed_objects: self.heap.freed_objects,
                    freed_bytes: self.heap.freed_bytes,
                    freed_los_objects: self.heap.freed_los_objects,
                    freed_los_bytes: self.heap.freed_los_bytes,
                    pause_times: self.heap.pause_times.to_vec(),
                }))
            }))
        }
    }
}