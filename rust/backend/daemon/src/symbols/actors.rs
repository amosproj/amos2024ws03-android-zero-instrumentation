// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{io, marker::PhantomData, sync::Arc};

use tantivy::{collector::{DocSetCollector, TopDocs}, doc, query::{BooleanQuery, QueryParser, TermQuery}, schema::{Field, Value}, Index, IndexWriter, TantivyDocument, Term};
use tokio_stream::{Stream, StreamExt};
use ractor::{cast, concurrency::{spawn, JoinHandle}, factory::{queues::DefaultQueue, routing::QueuerRouting, Factory, FactoryArguments, FactoryMessage, Job, JobOptions, WorkerBuilder, WorkerMessage, WorkerStartContext}, Actor, ActorCell, ActorProcessingErr, ActorRef, RpcReplyPort, SpawnErr, State, SupervisionEvent};

use super::{index::index, symbolizer::{symbols, Symbol}, walking::{all_symbol_files, SymbolFilePath}};

struct SymbolWithPath {
    symbol: Symbol,
    path: String,
}

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
            _: Self::Msg,
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
    type Arguments = (usize, ActorCell, ActorRef<SymbolWithPath>);
    type State = ActorRef<FactoryMessage<(), SymbolFilePath>>;
    type Msg = SymbolFilePath;
    
    async fn pre_start(
            &self,
            _: ActorRef<Self::Msg>,
            args: Self::Arguments,
        ) -> Result<Self::State, ActorProcessingErr> {
        let factory_def = Factory::<(), SymbolFilePath, ActorRef<SymbolWithPath>, SymbolFileParser, QueuerRouting<(), SymbolFilePath>, DefaultQueue::<(), SymbolFilePath>>::default();
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
            _: ActorRef<Self::Msg>,
            message: Self::Msg,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        Ok(cast!(state, FactoryMessage::Dispatch(Job { key: (), accepted: None, msg: message, options: JobOptions::default() }))?)
    }
    
    async fn post_stop(
            &self,
            _: ActorRef<Self::Msg>,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        Ok(cast!(state, FactoryMessage::DrainRequests)?)
    }

    async fn handle_supervisor_evt(
            &self,
            myself: ActorRef<Self::Msg>,
            message: SupervisionEvent,
            _: &mut Self::State,
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

struct SymbolFileParserBuilder(ActorRef<SymbolWithPath>);
impl WorkerBuilder<SymbolFileParser, ActorRef<SymbolWithPath>> for SymbolFileParserBuilder {
    fn build(&mut self, _: ractor::factory::WorkerId) -> (SymbolFileParser, ActorRef<SymbolWithPath>) {
        (SymbolFileParser, self.0.clone())
    }
}

struct SymbolFileParser;

impl Actor for SymbolFileParser {
    type Arguments = WorkerStartContext<(), SymbolFilePath, ActorRef<SymbolWithPath>>;
    type State = WorkerStartContext<(), SymbolFilePath, ActorRef<SymbolWithPath>>;
    type Msg = WorkerMessage<(), SymbolFilePath>;
    
    async fn pre_start(
            &self,
            _: ActorRef<Self::Msg>,
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
                let path = job.msg.path().to_string_lossy().into_owned();
                let handle = spawn(symbols(job.msg, tx));
                
                let mut stream = rx.into_stream();
                
                while let Some(symbol) = stream.next().await {
                    cast!(state.custom_start, SymbolWithPath { symbol, path: path.clone() })?;
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
    type State = (Field, Field, Field, Field, Arc<IndexWriter>);
    type Msg = SymbolWithPath;
    
    async fn pre_start(
            &self,
            _: ActorRef<Self::Msg>,
            args: Self::Arguments,
        ) -> Result<Self::State, ActorProcessingErr> {
        let symbol_name = args.index().schema().get_field("symbol_name")?;
        let symbol_offset = args.index().schema().get_field("symbol_offset")?;
        let library_path = args.index().schema().get_field("library_path")?;
        let symbol_name_extact = args.index().schema().get_field("symbol_name_exact")?;
        Ok((symbol_name, symbol_offset, library_path, symbol_name_extact, args))
    }
    
    async fn handle(
            &self,
            _: ActorRef<Self::Msg>,
            message: Self::Msg,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        state.4.add_document(doc!{
            state.0 => message.symbol.name.clone(),
            state.1 => message.symbol.offset,
            state.2 => message.path,
            state.3 => message.symbol.name,
        })?;
        
        Ok(())
    }
    
    async fn handle_supervisor_evt(
            &self,
            myself: ActorRef<Self::Msg>,
            message: SupervisionEvent,
            _: &mut Self::State,
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

pub struct SearchReq { pub query: String, pub limit: u64 }
pub type SearchRes = Result<Vec<shared::ziofa::Symbol>, io::Error>;

pub struct GetOffsetRequest { pub symbol_name: String, pub library_path: String }

pub enum SymbolActorMsg {
    ReIndex(RpcReplyPort<()>),
    Search(SearchReq, RpcReplyPort<SearchRes>),
    GetOffset(GetOffsetRequest, RpcReplyPort<Option<u64>>),
}

impl Actor for SymbolActor {
    type Msg = SymbolActorMsg;
    type Arguments = ();
    type State = Index;
    
    async fn pre_start(
            &self,
            _: ActorRef<Self::Msg>,
            _: Self::Arguments,
        ) -> Result<Self::State, ActorProcessingErr> {
        let index = index()?;
        Ok(index)
    }
    
    async fn handle(
            &self,
            _: ActorRef<Self::Msg>,
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
            SymbolActorMsg::Search(SearchReq { query, limit }, reply) => {
                let reader = state.reader()?;
                let searcher = reader.searcher();
                let symbol_name = state.schema().get_field("symbol_name")?;
                let symbol_offset = state.schema().get_field("symbol_offset")?;
                let library_path = state.schema().get_field("library_path")?;

                let query = match QueryParser::for_index(state, vec![symbol_name]).parse_query(&query) {
                    Ok(query) => query,
                    Err(e) => {
                        return Ok(reply.send(Err(io::Error::other(e)))?);
                    },
                };
                
                let results = searcher.search(&query, &TopDocs::with_limit(limit as usize))?.into_iter().map(|(_, address)| {
                    let doc : TantivyDocument = searcher.doc(address).map_err(io::Error::other)?;
                    let name = doc.get_first(symbol_name).and_then(|x| x.as_str()).ok_or_else(|| io::Error::other("expected str"))?;
                    let path = doc.get_first(library_path).and_then(|x| x.as_str()).ok_or_else(|| io::Error::other("expected str"))?;
                    let offset = doc.get_first(symbol_offset).and_then(|x| x.as_u64()).ok_or_else(|| io::Error::other("expected u64"))?;
                    
                    Ok::<_, io::Error>(shared::ziofa::Symbol { method: name.to_owned(), offset, path: path.to_owned() })
                }).collect::<Result<Vec<_>, _>>();
                
                reply.send(results)?;
            }
            SymbolActorMsg::GetOffset(GetOffsetRequest { symbol_name, library_path }, reply ) => {
                let reader = state.reader()?;
                let searcher = reader.searcher();
                let symbol_name_exact = state.schema().get_field("symbol_name_exact")?;
                let library_path_exact = state.schema().get_field("library_path")?;
                let symbol_offset = state.schema().get_field("symbol_offset")?;

                let symbol_query = TermQuery::new(Term::from_field_text(symbol_name_exact, &symbol_name), tantivy::schema::IndexRecordOption::Basic);
                let library_path_query = TermQuery::new(Term::from_field_text(library_path_exact, &library_path), tantivy::schema::IndexRecordOption::Basic);
                let query = BooleanQuery::intersection(vec![Box::new(symbol_query), Box::new(library_path_query)]);

                let result = searcher.search(&query, &DocSetCollector)?.into_iter().map(|address| {
                    let doc : TantivyDocument = searcher.doc(address).map_err(io::Error::other)?;
                    let offset = doc.get_first(symbol_offset).and_then(|x| x.as_u64()).ok_or_else(|| io::Error::other("expected u64"))?;
                    
                    Ok::<_, io::Error>(offset)
                }).next().transpose()?;
                
                reply.send(result)?;
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

async fn spawn_symbol_file_parser(destination: ActorRef<SymbolWithPath>, num: usize, supervisor: Option<ActorCell>) -> Result<ActorRef<SymbolFilePath>, SpawnErr> {
    let sup = supervisor.unwrap_or_else(|| destination.get_cell());
    
    let (actor_ref, _) = Actor::spawn(Some("symbol-file-parser-supervisor".to_owned()), SymbolFileParserProxy, (num, sup, destination)).await?;
    
    Ok(actor_ref)
}

async fn spawn_symbol_indexer(writer: Arc<IndexWriter>) -> Result<(ActorRef<SymbolWithPath>, JoinHandle<()>), SpawnErr> {
    let (actor_ref, handle) = Actor::spawn(Some("symbol-inderer".to_owned()), SymbolIndexer, writer).await?;
    
    Ok((actor_ref, handle))
}
