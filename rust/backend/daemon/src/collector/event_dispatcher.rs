// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use async_broadcast::Sender;
use ractor::Actor;
use shared::events::Event;
use tonic::Status;

pub struct EventDispatcher;

pub struct EventDispatcherState {
    destination: Sender<Result<Event, Status>>,
}

impl EventDispatcherState {
    pub fn new(destination: Sender<Result<Event, Status>>) -> Self {
        Self { destination }
    }
}

impl Actor for EventDispatcher {
    type Msg = Event;
    type State = EventDispatcherState;
    type Arguments = EventDispatcherState;

    async fn pre_start(
        &self,
        _: ractor::ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(args)
    }

    async fn handle(
        &self,
        _: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        state.destination.broadcast_direct(Ok(message)).await?;

        Ok(())
    }
}
