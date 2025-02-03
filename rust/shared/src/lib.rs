// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::time::Duration;

use events::{event::EventData, log_event::LogEventData, time_series_event::EventKind, Event, LogEvent};


#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

/*
 * List all proto files here.
 */
pub mod ziofa {
    tonic::include_proto!("ziofa");
}
pub mod config {
    tonic::include_proto!("config");
}
pub mod events {
    tonic::include_proto!("events");
}
pub mod processes {
    tonic::include_proto!("processes");
}
pub mod symbols {
    tonic::include_proto!("symbols");
}
pub mod google {
    pub mod protobuf_internal {
        tonic::include_proto!("google.protobuf");
    }
    pub mod protobuf {
        pub use super::protobuf_internal::{Duration, Timestamp};
        pub type Empty = ();
    }
}

impl From<Duration> for google::protobuf::Duration {
    fn from(value: Duration) -> Self {
        google::protobuf::Duration {
            seconds: value.as_secs() as i64,
            nanos: value.subsec_nanos() as i32,
        }
    }
}

impl From<google::protobuf::Duration> for Duration {
    fn from(value: google::protobuf::Duration) -> Self {
        Duration::from_secs(value.seconds as u64) + Duration::from_nanos(value.nanos as u64)
    }
}

impl From<Duration> for google::protobuf::Timestamp {
    fn from(value: Duration) -> Self {
        google::protobuf::Timestamp {
            seconds: value.as_secs() as i64,
            nanos: value.subsec_nanos() as i32,
        }
    }
}

impl From<google::protobuf::Timestamp> for Duration {
    fn from(value: google::protobuf::Timestamp) -> Self {
        Duration::from_secs(value.seconds as u64) + Duration::from_nanos(value.nanos as u64)
    }
}

impl<'a> From<&'a Event> for EventKind {
    fn from(value: &'a Event) -> Self {
        let Some(EventData::Log(log_event)) = value.event_data.as_ref() else {
            return EventKind::Undefined
        };
        
        log_event.into()
    }
}

impl<'a> From<&'a LogEvent> for EventKind {
    fn from(value: &'a LogEvent) -> Self {
        let Some(data) = value.log_event_data.as_ref() else {
            return EventKind::Undefined
        };
        
        match data {
            LogEventData::JniReferences(_) => EventKind::JniReferences,
            LogEventData::Signal(_) => EventKind::Signal,
            LogEventData::GarbageCollect(_) => EventKind::GarbageCollect,
            LogEventData::FileDescriptorChange(_) => EventKind::FileDescriptorChange,
            LogEventData::Blocking(_) => EventKind::Blocking,
            LogEventData::Write(_) => EventKind::Write,
        }
    }
}