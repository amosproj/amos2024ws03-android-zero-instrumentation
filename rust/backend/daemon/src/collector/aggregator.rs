// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use ractor::{
    cast,
    concurrency::{Duration, JoinHandle},
    Actor, ActorProcessingErr, ActorRef,
};
use shared::events::{
    event::EventData, time_series_event::{EventKind, TimeSeriesData}, Event, TimeSeriesEvent as ZioTimeSeriesEvent
};
use tokio::time;

use crate::{collector::time_series::TimeSeries, constants::TIMESERIES_LENGTH};

pub struct Aggregator;
impl Aggregator {
    fn convert_map_to_prototype(
        time_series_map: HashMap<u32, TimeSeries>,
    ) -> HashMap<u32, TimeSeriesData> {
        let mut map = HashMap::<u32, TimeSeriesData>::with_capacity(time_series_map.len());
        for (id, time_series) in time_series_map {
            map.insert(id, time_series.into());
        }
        map
    }
}

impl Default for Aggregator {
    fn default() -> Self {
        Aggregator
    }
}

pub struct AggregatorState {
    event_type: EventKind,
    event_count_map: HashMap<u32, u64>, // map pid to count
    timeframe: Duration,
    event_actor: ActorRef<Event>,
    timer: Option<JoinHandle<()>>,
    time_series_map: HashMap<u32, TimeSeries>,
}

pub struct AggregatorArguments {
    event_actor: ActorRef<Event>,
    timeframe: time::Duration,
    event_type_enum: EventKind,
}

impl AggregatorArguments {
    pub fn _new(
        event_actor: ActorRef<Event>,
        timeframe: Duration,
        event_type_enum: EventKind,
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
            event_count_map: HashMap::new(),
            timeframe: args.timeframe,
            event_actor: args.event_actor,
            timer: None,
            time_series_map: HashMap::new(),
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
        state.timer = Some(myself.send_interval(state.timeframe, || Event { event_data: None }));
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
        match msg.event_data {
            Some(EventData::Log(event)) => {
                let pid = event.context.unwrap().pid;

                let msg_event_type = EventKind::from(&event);

                if msg_event_type != state.event_type {
                    panic!(
                        "event type mismatch in Aggregator -> I was initialized with {:?}, but was send an {:?}",
                        state.event_type, msg_event_type
                    );
                }
                state.event_count_map.entry(pid).or_insert(1);
            }
            _ => {
                // event type is none -> timer was triggered -> send the metric
                for (key, value) in state.event_count_map.iter() {
                    if !state.time_series_map.contains_key(key) {
                        let mut new_ts = TimeSeries::new(TIMESERIES_LENGTH);
                        new_ts.append(*value);
                        state.time_series_map.insert(*key, new_ts);
                    } else {
                        state.time_series_map.get_mut(key).unwrap().append(*value);
                    }
                }

                //convert type for sending
                //events::time_series_event::TimeSeries

                let time_series = ZioTimeSeriesEvent {
                    event_kind: state.event_type.into(),
                    timeframe_ms: state.timeframe.as_millis() as u32,
                    time_series_map: Self::convert_map_to_prototype(state.time_series_map.clone()),
                };

                cast!(
                    state.event_actor,
                    Event {
                        event_data: Some(EventData::TimeSeries(time_series))
                    }
                )
                .map_err(|_| ActorProcessingErr::from("Failed to send metric to event actor"))?;
                for (_, value) in state.event_count_map.iter_mut() {
                    *value = 0;
                }
            }
        }
        Ok(())
    }
}
