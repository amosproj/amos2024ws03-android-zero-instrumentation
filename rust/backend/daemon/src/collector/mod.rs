// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use backend_common::{JNICall, JNIMethodName, SysSendmsgCall, SysSigquitCall, VfsWriteCall};
use shared::ziofa::event::EventType;
use shared::ziofa::jni_references_event::JniMethodName;
use shared::ziofa::log::EventData;
use shared::ziofa::{Event, JniReferencesEvent, Log, SysSendmsgEvent, SysSigquitEvent, VfsWriteEvent};

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
            event_type: Some(EventType::Log(Log {
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
            event_type: Some(EventType::Log(Log {
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
            event_type: Some(EventType::Log(Log {
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
            event_type: Some(EventType::Log(Log {
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