// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use crate::collector::time_series::TimeSeries;
use crate::constants::TIMESERIES_LENGTH;
use ractor::concurrency::{Duration, JoinHandle};
use ractor::{cast, Actor, ActorProcessingErr, ActorRef};
use shared::ziofa::event::EventType;
use shared::ziofa::log_event::EventData;
use shared::ziofa::time_series_event::EventTypeEnum;
use shared::ziofa::time_series_event::TimeSeries as ZioTimeSeries;
use shared::ziofa::{Event, TimeSeriesEvent as ZioTimeSeriesEvent};
use std::collections::HashMap;
use tokio::time;

pub struct Aggregator;
impl Aggregator {
    fn convert_map_to_prototype(
        time_series_map: HashMap<u32, TimeSeries>,
    ) -> HashMap<u32, ZioTimeSeries> {
        let mut map = HashMap::<u32, ZioTimeSeries>::with_capacity(time_series_map.len());
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
    event_type: EventTypeEnum,
    event_count_map: HashMap<u32, u64>, // map pid to count
    timeframe: Duration,
    event_actor: ActorRef<Event>,
    timer: Option<JoinHandle<()>>,
    time_series_map: HashMap<u32, TimeSeries>,
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
                let pid = match event.event_data.clone() {
                    Some(EventData::VfsWrite(item)) => item.pid,
                    Some(EventData::SysSendmsg(item)) => item.pid,
                    Some(EventData::JniReferences(item)) => item.pid,
                    Some(EventData::SysSigquit(item)) => item.pid,
                    Some(EventData::Gc(item)) => item.pid,
                    _ => {
                        panic!("unexpected event type");
                    }
                };

                let msg_event_type = EventTypeEnum::from(event);

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
                //ziofa::time_series_event::TimeSeries

                let time_series = ZioTimeSeriesEvent {
                    event_type_enum: state.event_type.into(),
                    timeframe_ms: state.timeframe.as_millis() as u32,
                    time_series_map: Self::convert_map_to_prototype(state.time_series_map.clone()),
                };

                cast!(
                    state.event_actor,
                    Event {
                        event_type: Some(EventType::TimeSeries(time_series))
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
