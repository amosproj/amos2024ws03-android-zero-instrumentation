// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::ziofa::event::EventType;
use crate::ziofa::log::EventData;
use crate::ziofa::metric::EventTypeEnum;
use crate::ziofa::{Event, Log};

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

impl TryInto<EventTypeEnum> for Event {
    type Error = ();

    fn try_into(self) -> Result<EventTypeEnum, ()> {
        match self {
            Event {
                event_type:
                    Some(EventType::Log(Log {
                        event_data: Some(EventData::VfsWrite(_)),
                    })),
            } => Ok(EventTypeEnum::VfsWriteEvent),
            Event {
                event_type:
                    Some(EventType::Log(Log {
                        event_data: Some(EventData::SysSendmsg(_)),
                    })),
            } => Ok(EventTypeEnum::JniReferencesEvent),
            Event {
                event_type:
                    Some(EventType::Log(Log {
                        event_data: Some(EventData::JniReferences(_)),
                    })),
            } => Ok(EventTypeEnum::SysSendmsgEvent),
            _ => Err(()),
        }
    }
}

impl From<Log> for EventTypeEnum {

    fn from(value: Log) -> Self {
        match value {
            Log {
                event_data: Some(EventData::VfsWrite(_)),
            } => EventTypeEnum::VfsWriteEvent,
            Log {
                event_data: Some(EventData::SysSendmsg(_)),
            } => EventTypeEnum::SysSendmsgEvent,
            Log {
                event_data: Some(EventData::JniReferences(_)),
            } => EventTypeEnum::JniReferencesEvent,
            _ => panic!()
        }
    }
}
