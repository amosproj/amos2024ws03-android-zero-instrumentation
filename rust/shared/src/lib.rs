// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

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
    pub mod protobuf {
        tonic::include_proto!("google.protobuf");
    }
}

impl<'a> From<&'a Event> for EventKind {
    fn from(value: &'a Event) -> Self {
        let Some(EventData::LogEvent(log_event)) = value.event_data.as_ref() else {
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
            LogEventData::WriteEvent(_) => EventKind::Write,
            LogEventData::BlockingEvent(_) => EventKind::Blocking,
            LogEventData::JniReferencesEvent(_) => EventKind::JniReferences,
            LogEventData::SignalEvent(_) => EventKind::Signal,
            LogEventData::GarbageCollectEvent(_) => EventKind::GarbageCollect,
            LogEventData::FileDescriptorChangeEvent(_) => EventKind::FileDescriptorChange,
        }
    }
}