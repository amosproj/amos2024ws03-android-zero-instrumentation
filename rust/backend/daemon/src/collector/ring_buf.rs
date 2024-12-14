// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{io, marker::PhantomData};

use backend_common::TryFromRaw;
use ractor::{cast, Actor, ActorRef};
use shared::ziofa::Event;
use tokio::io::unix::AsyncFd;

use crate::registry::{RegistryGuard, RegistryItem, TypedRingBuffer};

use super::IntoEvent;



pub struct RingBufCollector<T>(PhantomData<T>);

impl<T> Default for RingBufCollector<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}


pub struct RingBufCollectorState<T> {
    map: AsyncFd<RegistryGuard<TypedRingBuffer<T>>>,
    event_actor: ActorRef<Event>,
}

pub struct RingBufCollectorArguments<T> {
    item: RegistryItem<TypedRingBuffer<T>>,
    event_actor:ActorRef<Event>
}

impl<T> RingBufCollectorArguments<T> {
    pub fn new(item: RegistryItem<TypedRingBuffer<T>>, event_actor: ActorRef<Event>) -> Self {
        Self {
            item,
            event_actor
        }
    }
}


impl<T> TryFrom<RingBufCollectorArguments<T>> for RingBufCollectorState<T> {
    type Error = io::Error;
    
    fn try_from(value: RingBufCollectorArguments<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            map: AsyncFd::new(value.item.take())?,
            event_actor: value.event_actor,
        })
    }
}

impl<T> Actor for RingBufCollector<T>
where T: TryFromRaw + IntoEvent + Send + Sync + 'static
{
    type Msg = ();
    type State = RingBufCollectorState<T>;
    type Arguments = RingBufCollectorArguments<T>;
    
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

        while let Some(item) = inner.next().map(T::into_event) {
            cast!(state.event_actor, item)?;
        }
        
        guard.clear_ready();
        
        Ok(cast!(myself, ())?)
    }
}