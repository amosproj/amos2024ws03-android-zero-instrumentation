// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use crate::collector::event_dispatcher::{EventDispatcher, EventDispatcherState};
use crate::collector::ring_buf::{RingBufCollector, RingBufCollectorArguments};
use crate::collector::IntoEvent;
use crate::registry::{EbpfEventRegistry, RegistryItem, TypedRingBuffer};
use backend_common::TryFromRaw;
use ractor::{Actor, ActorCell, ActorProcessingErr, ActorRef, SupervisionEvent};
use shared::ziofa::Event;
use tonic::Status;
use tracing::error;

#[derive(Clone, Copy)]
enum CollectorT {
    VfsWrite,
    SysSendmsg,
    JniCall,
    SysSigquit,
}

pub struct CollectorSupervisor;

pub struct CollectorSupervisorState {
    registry: EbpfEventRegistry,
    collectors: CollectorRefs,
    events: ActorRef<Event>,
}

struct CollectorRefs {
    collectors: HashMap<ActorCell, CollectorT>,
}

impl CollectorRefs {
    async fn from_registry(
        registry: EbpfEventRegistry,
        event_actor: ActorRef<Event>,
        supervisor: ActorCell,
    ) -> Result<Self, ActorProcessingErr> {
        let mut this = Self {
            collectors: HashMap::new(),
        };
        this.start_all(&registry, &event_actor, &supervisor).await?;
        Ok(this)
    }
    fn who_is(&self, cell: &ActorCell) -> Option<CollectorT> {
        self.collectors.get(cell).copied()
    }
    fn remove(&mut self, cell: &ActorCell) -> Option<CollectorT> {
        self.collectors.remove(cell)
    }
    async fn start_all(
        &mut self,
        registry: &EbpfEventRegistry,
        event_actor: &ActorRef<Event>,
        supervisor: &ActorCell,
    ) -> Result<(), ActorProcessingErr> {
        for who in [
            CollectorT::VfsWrite,
            CollectorT::SysSendmsg,
            CollectorT::JniCall,
            CollectorT::SysSigquit,
        ] {
            self.start(who, registry, event_actor, supervisor).await?;
        }
        Ok(())
    }
    async fn start(
        &mut self,
        who: CollectorT,
        registry: &EbpfEventRegistry,
        event_actor: &ActorRef<Event>,
        supervisor: &ActorCell,
    ) -> Result<(), ActorProcessingErr> {
        let actor_ref = match who {
            CollectorT::VfsWrite => {
                start_collector(
                    registry.vfs_write_events.clone(),
                    event_actor.clone(),
                    supervisor.clone(),
                )
                .await?
            }
            CollectorT::SysSendmsg => {
                start_collector(
                    registry.sys_sendmsg_events.clone(),
                    event_actor.clone(),
                    supervisor.clone(),
                )
                .await?
            }
            CollectorT::JniCall => {
                start_collector(
                    registry.jni_ref_calls.clone(),
                    event_actor.clone(),
                    supervisor.clone(),
                )
                .await?
            }
            CollectorT::SysSigquit => {
                start_collector(
                    registry.sys_sigquit_events.clone(),
                    event_actor.clone(),
                    supervisor.clone(),
                )
                .await?
            }
        };
        self.collectors.insert(actor_ref.get_cell(), who);
        Ok(())
    }
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
    type Arguments = CollectorSupervisorArguments;
    type Msg = ();
    type State = CollectorSupervisorState;

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
        let collectors =
            CollectorRefs::from_registry(args.registry.clone(), events.clone(), myself.get_cell())
                .await?;

        Ok(CollectorSupervisorState {
            registry: args.registry.clone(),
            collectors,
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
            if let Some(who) = state.collectors.who_is(&actor_cell) {
                error!("Collector {:?} failed with {:?}", actor_cell, error);
                let _ = state.collectors.remove(&actor_cell);
                state
                    .collectors
                    .start(who, &state.registry, &state.events, &myself.get_cell())
                    .await?;
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

async fn start_collector<T>(
    item: RegistryItem<TypedRingBuffer<T>>,
    event_actor: ActorRef<Event>,
    supervisor: ActorCell,
) -> Result<ActorRef<()>, ActorProcessingErr>
where
    T: TryFromRaw + IntoEvent + Send + Sync + 'static,
{
    let (actor_ref, _) = Actor::spawn_linked(
        None,
        RingBufCollector::default(),
        RingBufCollectorArguments::new(item, event_actor),
        supervisor,
    )
    .await?;
    Ok(actor_ref)
}
