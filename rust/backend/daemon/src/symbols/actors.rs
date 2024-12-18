use std::{default, marker::PhantomData, sync::{Arc, RwLock}, time::Duration};

use futures::FutureExt;
use libc::stat;
use serde::de;
use tantivy::{doc, schema::Field, Index, IndexWriter, TantivyDocument};
use tokio_stream::{Stream, StreamExt};
use ractor::{cast, concurrency::{interval, sleep, spawn, JoinHandle}, factory::{queues::{DefaultQueue, Queue}, routing::QueuerRouting, Factory, FactoryArguments, FactoryLifecycleHooks, FactoryMessage, Job, JobOptions, WorkerBuilder, WorkerMessage, WorkerStartContext}, Actor, ActorCell, ActorProcessingErr, ActorRef, RpcReplyPort, SpawnErr, State, SupervisionEvent};

use super::{index::index, symbolizer::{symbols, Symbol}, walking::{all_symbol_files, SymbolFilePath}};

struct SymbolFilePathCollector<S>(PhantomData<S>);

// SAFETY: `S` is only present as PhantomData, so it doesn't actually affect the Actor itself.
unsafe impl<S> Sync for SymbolFilePathCollector<S> where S: Stream + State {}

impl<S> Actor for SymbolFilePathCollector<S>
where S: Stream<Item = SymbolFilePath> + State + Unpin {
    type Arguments = (S, ActorRef<SymbolFilePath>);
    type State = (S, ActorRef<SymbolFilePath>);
    type Msg = ();
    
    async fn pre_start(
            &self,
            myself: ActorRef<Self::Msg>,
            args: Self::Arguments,
        ) -> Result<Self::State, ActorProcessingErr> {
        cast!(myself, ())?;
        Ok(args)
    }
    
    async fn handle(
            &self,
            myself: ActorRef<Self::Msg>,
            message: Self::Msg,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        if let Some(next) = state.0.next().await {
            cast!(state.1, next)?;
            cast!(myself, ())?;
        } else {
            myself.stop(None);
        }
        Ok(())
    }
}

struct SymbolFileParserProxy;

impl Actor for SymbolFileParserProxy {
    type Arguments = (usize, ActorCell, ActorRef<Symbol>);
    type State = ActorRef<FactoryMessage<(), SymbolFilePath>>;
    type Msg = SymbolFilePath;
    
    async fn pre_start(
            &self,
            myself: ActorRef<Self::Msg>,
            args: Self::Arguments,
        ) -> Result<Self::State, ActorProcessingErr> {
        let factory_def = Factory::<(), SymbolFilePath, ActorRef<Symbol>, SymbolFileParser, QueuerRouting<(), SymbolFilePath>, DefaultQueue::<(), SymbolFilePath>>::default();
        let factory_args = FactoryArguments::builder()
            .num_initial_workers(args.0)
            .worker_builder(Box::new(SymbolFileParserBuilder(args.2.clone())))
            .queue(DefaultQueue::default())
            .router(QueuerRouting::default())
            .build();
            
        let (actor_ref, _) = Actor::spawn_linked(Some("symbol-file-parser-factory".to_owned()), factory_def, factory_args, args.2.get_cell()).await?;

        Ok(actor_ref)
    }
    
    async fn handle(
            &self,
            myself: ActorRef<Self::Msg>,
            message: Self::Msg,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        Ok(cast!(state, FactoryMessage::Dispatch(Job { key: (), accepted: None, msg: message, options: JobOptions::default() }))?)
    }
    
    async fn post_stop(
            &self,
            myself: ActorRef<Self::Msg>,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        Ok(cast!(state, FactoryMessage::DrainRequests)?)
    }

    async fn handle_supervisor_evt(
            &self,
            myself: ActorRef<Self::Msg>,
            message: SupervisionEvent,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        match message {
            SupervisionEvent::ActorFailed(_,  reason) => {
                myself.stop(Some(reason.to_string()));
            }
            SupervisionEvent::ActorTerminated(cell, _, _) => {
                myself.drain()?;
            }
            _ => {}
        }
        Ok(())
    }
}

struct SymbolFileParserBuilder(ActorRef<Symbol>);
impl WorkerBuilder<SymbolFileParser, ActorRef<Symbol>> for SymbolFileParserBuilder {
    fn build(&mut self, wid: ractor::factory::WorkerId) -> (SymbolFileParser, ActorRef<Symbol>) {
        (SymbolFileParser, self.0.clone())
    }
}

struct SymbolFileParser;

impl Actor for SymbolFileParser {
    type Arguments = WorkerStartContext<(), SymbolFilePath, ActorRef<Symbol>>;
    type State = WorkerStartContext<(), SymbolFilePath, ActorRef<Symbol>>;
    type Msg = WorkerMessage<(), SymbolFilePath>;
    
    async fn pre_start(
            &self,
            myself: ActorRef<Self::Msg>,
            args: Self::Arguments,
        ) -> Result<Self::State, ActorProcessingErr> {
        Ok(args)
    }
    
    async fn handle(
            &self,
            _: ActorRef<Self::Msg>,
            message: Self::Msg,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        match message {
            WorkerMessage::FactoryPing(instant) =>
                Ok(cast!(state.factory, FactoryMessage::WorkerPong(state.wid, instant.elapsed()))?),
            WorkerMessage::Dispatch(job) => {
                let (tx, rx) = flume::bounded(0);
                let handle = spawn(symbols(job.msg, tx));
                
                let mut stream = rx.into_stream();
                
                while let Some(symbol) = stream.next().await {
                    cast!(state.custom_start, symbol)?;
                }
                
                handle.await??;
                
                cast!(state.factory, FactoryMessage::Finished(state.wid, ()))?;
                
                Ok(())
            }
        }
    }   
}

struct SymbolIndexer;

impl Actor for SymbolIndexer {
    type Arguments = Arc<IndexWriter>;
    type State = (Field, Field, Arc<IndexWriter>);
    type Msg = Symbol;
    
    async fn pre_start(
            &self,
            _: ActorRef<Self::Msg>,
            args: Self::Arguments,
        ) -> Result<Self::State, ActorProcessingErr> {
        let symbol_name = args.index().schema().get_field("symbol_name")?;
        let symbol_offset = args.index().schema().get_field("symbol_offset")?;
        Ok((symbol_name, symbol_offset, args))
    }
    
    async fn handle(
            &self,
            _: ActorRef<Self::Msg>,
            message: Self::Msg,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        state.2.add_document(doc!{
            state.0 => message.name,
            state.1 => message.offset,
        })?;
        
        Ok(())
    }
    
    async fn handle_supervisor_evt(
            &self,
            myself: ActorRef<Self::Msg>,
            message: SupervisionEvent,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        match message {
            SupervisionEvent::ActorFailed(_,  reason) => {
                myself.stop(Some(reason.to_string()));
            }
            SupervisionEvent::ActorTerminated(_, _, _) => {
                myself.drain()?;
            }
            _ => {}
        }
        Ok(())
    }
}

pub struct SymbolActor;

impl SymbolActor {
    pub async fn spawn() -> Result<ActorRef<SymbolActorMsg>, SpawnErr> {
        let (myself, _) = Actor::spawn(None, SymbolActor, ()).await?;
        Ok(myself)
    }
}

pub enum SymbolActorMsg {
    ReIndex(RpcReplyPort<()>),
}

impl Actor for SymbolActor {
    type Msg = SymbolActorMsg;
    type Arguments = ();
    type State = Index;
    
    async fn pre_start(
            &self,
            myself: ActorRef<Self::Msg>,
            args: Self::Arguments,
        ) -> Result<Self::State, ActorProcessingErr> {
        let index = index()?;
        Ok(index)
    }
    
    async fn handle(
            &self,
            myself: ActorRef<Self::Msg>,
            message: Self::Msg,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        
        match message {
            SymbolActorMsg::ReIndex(reply) => {
                let writer = Arc::new(state.writer::<TantivyDocument>(50_000_000)?);
                let (symbol_indexer, handle) = spawn_symbol_indexer(writer.clone()).await?;
                let symbol_parser = spawn_symbol_file_parser(symbol_indexer, 4, None).await?;
                let _ = spawn_symbol_file_path_collector(symbol_parser, None).await?;
                
                handle.await?;
                
                Arc::into_inner(writer).expect("strong count should be 1").commit()?;
                
                reply.send(())?;
            }
        }
        
        Ok(())


    }
}


async fn spawn_symbol_file_path_collector(destination: ActorRef<SymbolFilePath>, supervisor: Option<ActorCell>) -> Result<ActorCell, SpawnErr> {
    let sup = supervisor.unwrap_or_else(|| destination.get_cell());
    let symbol_files = Box::pin(all_symbol_files());
    let (actor_ref, _) = Actor::spawn_linked(Some("symbol-file-path-collector".to_owned()), SymbolFilePathCollector(PhantomData), (symbol_files, destination), sup).await?;
    
    Ok(actor_ref.get_cell())
}

async fn spawn_symbol_file_parser(destination: ActorRef<Symbol>, num: usize, supervisor: Option<ActorCell>) -> Result<ActorRef<SymbolFilePath>, SpawnErr> {
    let sup = supervisor.unwrap_or_else(|| destination.get_cell());
    
    let (actor_ref, _) = Actor::spawn(Some("symbol-file-parser-supervisor".to_owned()), SymbolFileParserProxy, (num, sup, destination)).await?;
    
    Ok(actor_ref)
}

async fn spawn_symbol_indexer(writer: Arc<IndexWriter>) -> Result<(ActorRef<Symbol>, JoinHandle<()>), SpawnErr> {
    let (actor_ref, handle) = Actor::spawn(Some("symbol-inderer".to_owned()), SymbolIndexer, writer).await?;
    
    Ok((actor_ref, handle))
}
