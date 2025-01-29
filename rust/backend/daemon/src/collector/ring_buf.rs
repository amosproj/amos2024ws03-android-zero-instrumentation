// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::io;

use ractor::{cast, Actor, ActorRef};
use shared::ziofa::Event;
use tokio::io::unix::AsyncFd;

use super::IntoEvent;
use crate::registry::{OwnedRingBuf, RegistryGuard, RegistryItem};

pub struct RingBufCollector;

pub struct RingBufCollectorState {
    map: AsyncFd<RegistryGuard<OwnedRingBuf>>,
    event_actor: ActorRef<Event>,
}

pub struct RingBufCollectorArguments {
    item: RegistryItem<OwnedRingBuf>,
    event_actor: ActorRef<Event>,
}

impl RingBufCollectorArguments {
    pub fn new(item: RegistryItem<OwnedRingBuf>, event_actor: ActorRef<Event>) -> Self {
        Self { item, event_actor }
    }
}

impl TryFrom<RingBufCollectorArguments> for RingBufCollectorState {
    type Error = io::Error;

    fn try_from(value: RingBufCollectorArguments) -> Result<Self, Self::Error> {
        Ok(Self {
            map: AsyncFd::new(value.item.take())?,
            event_actor: value.event_actor,
        })
    }
}

impl Actor for RingBufCollector {
    type Msg = ();
    type State = RingBufCollectorState;
    type Arguments = RingBufCollectorArguments;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        cast!(myself, ())?;
        Ok(args.try_into()?)
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        _: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let mut guard = state.map.readable_mut().await?;
        let inner = guard.get_inner_mut();

        while let Some(item) = inner.next().map(IntoEvent::into_event) {
            cast!(state.event_actor, item)?;
        }

        guard.clear_ready();

        Ok(cast!(myself, ())?)
    }
}
