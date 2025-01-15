// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use ractor::concurrency::{Duration, JoinHandle};
use ractor::{cast, Actor, ActorProcessingErr, ActorRef};
use shared::ziofa::event::EventType;
use shared::ziofa::time_series_event::EventTypeEnum;
use shared::ziofa::{Event, TimeSeriesEvent as ZioTimeSeries};
use tokio::time;
use crate::collector::time_series::TimeSeries;
use crate::constants::TIMESERIES_LENGTH;

pub struct Aggregator;

impl Default for Aggregator {
    fn default() -> Self {
        Aggregator
    }
}

pub struct AggregatorState {
    event_type: EventTypeEnum,
    event_count: u32,
    timeframe: Duration,
    event_actor: ActorRef<Event>,
    timer: Option<JoinHandle<()>>,
    timeseries: TimeSeries
}

pub struct AggregatorArguments {
    event_actor: ActorRef<Event>,
    timeframe: time::Duration,
    event_type_enum: EventTypeEnum,
}

impl AggregatorArguments {
    pub fn _new(
        event_actor: ActorRef<Event>,
        timeframe: Duration,
        event_type_enum: EventTypeEnum,
    ) -> Self {
        AggregatorArguments {
            event_actor,
            timeframe,
            event_type_enum,
        }
    }
}

impl TryFrom<AggregatorArguments> for AggregatorState {
    type Error = ActorProcessingErr;
    fn try_from(args: AggregatorArguments) -> Result<AggregatorState, Self::Error> {
        Ok(AggregatorState {
            event_type: args.event_type_enum,
            event_count: 0,
            timeframe: args.timeframe,
            event_actor: args.event_actor,
            timer: None,
            timeseries: TimeSeries::new(TIMESERIES_LENGTH),
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
                let msg_event_type = EventTypeEnum::from(event);

                if msg_event_type != state.event_type {
                    panic!(
                        "event type mismatch in Aggregator -> I was initialized with {:?}, but was send an {:?}",
                        state.event_type, msg_event_type
                    );
                }
                state.event_count += 1;
            }
            _ => {
                // event type is none -> timer was triggered -> send the metric
                state.timeseries.append(state.event_count as u64);
                let time_series = ZioTimeSeries {
                    event_type_enum: state.event_type.into(),
                    timeframe_ms: state.timeframe.as_millis() as u32,
                    data: state.timeseries.as_array(),
                };
                
                cast!(
                    state.event_actor,
                    Event {
                        event_type: Some(EventType::TimeSeries(time_series))
                    }
                )
                .map_err(|_| ActorProcessingErr::from("Failed to send metric to event actor"))?;
                state.event_count = 0;
            }
        }
        Ok(())
    }
}
