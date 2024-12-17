// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use ractor::concurrency::{Duration, JoinHandle};
use ractor::{cast, Actor, ActorProcessingErr, ActorRef};
use shared::ziofa::event::EventType;
use shared::ziofa::metric::EventTypeEnum;
use shared::ziofa::{Event, Metric};
use tokio::time;

pub struct Aggregator;

pub struct AggregatorState {
    event_type: EventTypeEnum,
    event_count: u32,
    timeframe: Duration,
    event_actor: ActorRef<Event>,
    timer: Option<JoinHandle<()>>,
}

pub struct AggregatorArguments {
    event_actor: ActorRef<Event>,
    timeframe: time::Duration,
    event_type: EventTypeEnum,
}


impl TryFrom<AggregatorArguments> for AggregatorState {
    type Error = ActorProcessingErr;
    fn try_from(args: AggregatorArguments) -> Result<AggregatorState, Self::Error> {
        Ok(AggregatorState {
            event_type: args.event_type,
            event_count: 0,
            timeframe: args.timeframe,
            event_actor: args.event_actor,
            timer: None,
        })
    }
}


impl Actor for Aggregator {
    type Msg = Event;
    type State = AggregatorState;
    type Arguments = AggregatorArguments;

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Self::State::try_from(args)
    }

    async fn post_start(
        &self,
        myself: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        state.timer = Some(myself.send_interval(state.timeframe, || Event { event_type: None }));
        Ok(())
    }

    async fn post_stop(
        &self,
        _: ActorRef<Self::Msg>,
        state: &mut AggregatorState,
    ) -> Result<(), ActorProcessingErr> {
        if let Some(timer) = state.timer.take() {
            timer.abort();
        }
        Ok(())
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut AggregatorState,
    ) -> Result<(), ActorProcessingErr> {
        match msg.event_type {
            Some(EventType::Log(event)) => {
                let msg_event_type = EventTypeEnum::try_from(event).unwrap();

                if msg_event_type != state.event_type {
                    panic!(
                        "event type mismatch -> I was initialized with {:?}, but was send an {:?}",
                        state.event_type, msg_event_type
                    );
                }
                state.event_count += 1;
            }
            _ => {
                // event type is none -> timer was triggered -> send the metric
                let metric = Metric {
                    timeframe_ms: state.timeframe.as_millis() as u32,
                    event_count: state.event_count,
                    event_type_enum: state.event_type.into(),
                };
                cast!(
                    state.event_actor,
                    Event {
                        event_type: Some(EventType::Metric(metric))
                    }
                )
                    .map_err(|_| ActorProcessingErr::from("Failed to send metric to event actor"))?;
                state.event_count = 0;
            }
        }
        Ok(())
    }
}
