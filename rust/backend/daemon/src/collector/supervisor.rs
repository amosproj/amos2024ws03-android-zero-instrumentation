// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use ractor::{Actor, ActorCell, ActorProcessingErr, ActorRef, SupervisionEvent};
use shared::events::Event;
use tonic::Status;
use tracing::error;

use crate::{
    collector::{
        event_dispatcher::{EventDispatcher, EventDispatcherState},
        ring_buf::{RingBufCollector, RingBufCollectorArguments},
    },
    registry::{EbpfEventRegistry, OwnedRingBuf, RegistryItem},
};

pub struct CollectorSupervisor;

pub struct CollectorSupervisorState {
    registry: EbpfEventRegistry,
    event_colletor: ActorCell,
    events: ActorRef<Event>,
}

pub struct CollectorSupervisorArguments {
    registry: EbpfEventRegistry,
    sender: async_broadcast::Sender<Result<Event, Status>>,
}

impl CollectorSupervisorArguments {
    pub fn new(
        registry: EbpfEventRegistry,
        sender: async_broadcast::Sender<Result<Event, Status>>,
    ) -> Self {
        Self { registry, sender }
    }
}

impl Actor for CollectorSupervisor {
    type Msg = ();
    type State = CollectorSupervisorState;
    type Arguments = CollectorSupervisorArguments;

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let (events, _) = Actor::spawn_linked(
            None,
            EventDispatcher,
            EventDispatcherState::new(args.sender),
            myself.get_cell(),
        )
        .await?;
        let event_colletor = start_collector(
            args.registry.events.clone(),
            events.clone(),
            myself.get_cell(),
        )
        .await?
        .get_cell();

        Ok(CollectorSupervisorState {
            registry: args.registry.clone(),
            event_colletor,
            events,
        })
    }

    async fn handle_supervisor_evt(
        &self,
        myself: ActorRef<Self::Msg>,
        message: ractor::SupervisionEvent,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let SupervisionEvent::ActorFailed(actor_cell, error) = message {
            if actor_cell == state.event_colletor {
                error!("Collector {:?} failed with {:?}", actor_cell, error);
                state.event_colletor = start_collector(
                    state.registry.events.clone(),
                    state.events.clone(),
                    myself.get_cell(),
                )
                .await?
                .get_cell();
                Ok(())
            } else {
                Err(ActorProcessingErr::from(format!(
                    "Fatal {:?} failed with {:?}",
                    actor_cell, error
                )))
            }
        } else {
            Ok(())
        }
    }
}

async fn start_collector(
    item: RegistryItem<OwnedRingBuf>,
    event_actor: ActorRef<Event>,
    supervisor: ActorCell,
) -> Result<ActorRef<()>, ActorProcessingErr>
where
{
    let (actor_ref, _) = Actor::spawn_linked(
        None,
        RingBufCollector,
        RingBufCollectorArguments::new(item, event_actor),
        supervisor,
    )
    .await?;
    Ok(actor_ref)
}
