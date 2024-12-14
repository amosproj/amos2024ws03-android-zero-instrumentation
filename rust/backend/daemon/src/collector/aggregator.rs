use crate::collector::IntoEvent;
use crate::registry::{RegistryGuard, RegistryItem, TypedRingBuffer};
use backend_common::TryFromRaw;
use ractor::{cast, Actor, ActorProcessingErr, ActorRef};
use shared::ziofa::Event;
use std::io;
use std::marker::PhantomData;
use tokio::io::unix::AsyncFd;

pub struct Aggregator<T>(PhantomData<T>);

impl<T> Default for Aggregator<T> {
    fn default() -> Self {Self(PhantomData)}
}


pub struct AggregatorState<T> {
    map: AsyncFd<RegistryGuard<TypedRingBuffer<T>>>,
    event_actor: ActorRef<Event>,
}

pub struct AggregatorArguments<T> {
    item: RegistryItem<TypedRingBuffer<T>>,
    event_actor: ActorRef<Event>,
}

impl<T> AggregatorArguments<T> {
    pub fn new(item: RegistryItem<TypedRingBuffer<T>>, event_actor: ActorRef<Event>) -> Self {
        Self { item, event_actor }
    }
}

impl<T> TryFrom<AggregatorArguments<T>> for AggregatorState<T> {
    type Error = io::Error;

    fn try_from(value: AggregatorArguments<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            map: AsyncFd::new(value.item.take())?,
            event_actor: value.event_actor,
        })
    }
}

impl<T> Actor for Aggregator<T>
where
    T: TryFromRaw + IntoEvent + Send + Sync + 'static,
{
    type Msg = ();
    type State = AggregatorState<T>;
    type Arguments = AggregatorState<T>;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr>{
        cast!(myself, ())?;
        Ok(args.try_into()?)
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr>{
        let mut guard = state.map.readable_mut().await?;
        let inner = guard.get_inner_mut();

        while let Some(item) = inner.next().map(T::into_event) {
            cast!(state.event_actor, item)?;
        }

        guard.clear_ready();

        Ok(cast!(myself, ())?)
    }
}
