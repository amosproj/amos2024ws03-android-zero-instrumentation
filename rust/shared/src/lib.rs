// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::events::event::EventType;
use crate::events::log_event::EventData;
use crate::events::time_series_event::EventTypeEnum;
use crate::events::{Event, LogEvent};

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

pub mod counter {
    tonic::include_proto!("com.example.counter");
}

pub mod ziofa {
    tonic::include_proto!("ziofa");
}

pub mod config {
    tonic::include_proto!("config");
}

pub mod events {
    tonic::include_proto!("events");
}

impl TryInto<EventTypeEnum> for Event {
    type Error = ();

    fn try_into(self) -> Result<EventTypeEnum, ()> {
        match self {
            Event {
                event_type:
                    Some(EventType::Log(LogEvent {
                        event_data: Some(EventData::VfsWrite(_)),
                    })),
            } => Ok(EventTypeEnum::VfsWriteEvent),
            Event {
                event_type:
                    Some(EventType::Log(LogEvent {
                        event_data: Some(EventData::SysSendmsg(_)),
                    })),
            } => Ok(EventTypeEnum::SysSendmsgEvent),
            Event {
                event_type:
                    Some(EventType::Log(LogEvent {
                        event_data: Some(EventData::JniReferences(_)),
                    })),
            } => Ok(EventTypeEnum::JniReferencesEvent),Event {
                event_type:
                    Some(EventType::Log(LogEvent {
                        event_data: Some(EventData::SysSigquit(_)),
                    })),
            } => Ok(EventTypeEnum::SysSigquitEvent),
            Event{
                event_type:
                Some(EventType::Log(LogEvent {
                    event_data: Some(EventData::Gc(_)),
                                    }))
            } => Ok(EventTypeEnum::GcEvent),
            Event{
                event_type:
                Some(EventType::Log(LogEvent {
                    event_data: Some(EventData::SysFdTracking(_)),
                                    }))
            } => Ok(EventTypeEnum::SysFdTrackingEvent),
            _ => Err(()),
        }
    }
}

impl From<LogEvent> for EventTypeEnum {

    fn from(value: LogEvent) -> Self {
        match value {
            LogEvent {
                event_data: Some(EventData::VfsWrite(_)),
            } => EventTypeEnum::VfsWriteEvent,
            LogEvent {
                event_data: Some(EventData::SysSendmsg(_)),
            } => EventTypeEnum::SysSendmsgEvent,
            LogEvent {
                event_data: Some(EventData::JniReferences(_)),
            } => EventTypeEnum::JniReferencesEvent,
            _ => panic!()
        }
    }
}
