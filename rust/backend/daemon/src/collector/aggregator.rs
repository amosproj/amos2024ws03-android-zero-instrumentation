// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use ractor::{cast, Actor, ActorProcessingErr, ActorRef};
use shared::ziofa::event::EventType;
use shared::ziofa::metric::EventTypeEnum;
use shared::ziofa::{Event, Metric};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{self};

pub struct Aggregator;

pub struct AggregatorState {
    event_type: EventTypeEnum,
    event_count: Arc<Mutex<u32>>,
}

pub struct AggregatorArguments {
    event_actor: ActorRef<Event>,
    timeframe: time::Duration,
    event_type: Event,
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
        let count = Arc::new(Mutex::new(0));
        let l_count = count.clone();
        let event_type_enum: EventTypeEnum = args.event_type.try_into().expect("invalid event type (you are probably trying to aggregate an already aggregated event)");
            tokio::task::spawn(async move {
            loop {
                tokio::time::sleep(args.timeframe).await;
                let c_val = l_count.lock().await;

                let metric = Metric {
                    timeframe_ms: args.timeframe.clone().as_millis() as u32,
                    event_count: *c_val,
                    event_type_enum: event_type_enum.into(),
                };
                cast!(
                    args.event_actor,
                    Event {
                        event_type: Some(EventType::Metric(metric))
                    }
                )
                    .expect("Event couldn't be send to next actor");
            }
        });
        Ok(Self::State {
            event_type: event_type_enum,
            event_count: count,
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut AggregatorState,
    ) -> Result<(), ActorProcessingErr> {
        let AggregatorState {
            event_type: state_event_type,
            event_count: state_event_count,
        } = state;

        let msg_event_type: EventTypeEnum = msg.try_into().expect("Invalid event type in my stream (The event type during initialization was correct, but i got a mismatching item at runtime)");

        if *state_event_type != msg_event_type {
            return Err(ActorProcessingErr::from(
                "[Aggregator] Received event type does not match the type i was initialized with",
            ));
        }

        let mut event_count_mut = state_event_count.lock().await;
        *event_count_mut += 1;
        Ok(())
    }
}
