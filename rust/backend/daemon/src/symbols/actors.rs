use std::{fs::File, marker::PhantomData, path::PathBuf, pin::Pin, sync::Arc};

use futures::Stream;
use object::{write::elf::Sym, Object, ObjectSymbol, ReadCache};
use ractor::{cast, concurrency::JoinHandle, factory::{job::JobBuilder, queues::DefaultQueue, routing::QueuerRouting, Factory, FactoryArguments, FactoryMessage, Job, JobOptions, WorkerBuilder, WorkerMessage, WorkerStartContext}, Actor, ActorProcessingErr, ActorRef};
use symbolic_common::Name;
use symbolic_demangle::{Demangle, DemangleOptions};
use tantivy::{doc, schema::Field, TantivyDocument};
use tokio::task::spawn_blocking;
use tokio_stream::StreamExt;

use super::{symbolizer::{oat_symbols, Symbol}, walking::{all_symbol_files, SymbolFilePath}};


pub struct FileWalker;

impl Actor for FileWalker {
    type Arguments = ActorRef<SymbolFilePath>;
    type State = (Pin<Box<dyn Stream<Item = SymbolFilePath> + Send>>, ActorRef<SymbolFilePath>);
    type Msg = ();
    
    async fn pre_start(
            &self,
            myself: ractor::ActorRef<Self::Msg>,
            args: Self::Arguments,
        ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let s = Box::pin(all_symbol_files());
        cast!(myself, ())?;
        Ok((s, args))
    }
    
    async fn handle(
            &self,
            myself: ActorRef<Self::Msg>,
            _: Self::Msg,
            state: &mut Self::State,
        ) -> Result<(), ractor::ActorProcessingErr> {
        let (s, r) = state;
        
        if let Some(p) = s.next().await {
            cast!(r, p)?;
            cast!(myself, ())?;
        } else {
            myself.stop(None);
        }
        
        
        Ok(())
    }
}

pub struct SoParserWorker;

impl Actor for SoParserWorker {
    type Arguments = WorkerStartContext<(), PathBuf, ActorRef<Symbol>>;
    type State = WorkerStartContext<(), PathBuf, ActorRef<Symbol>>;
    type Msg = WorkerMessage<(), PathBuf>;

    async fn pre_start(
            &self,
            myself: ActorRef<Self::Msg>,
            args: Self::Arguments,
        ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(args)
    }
    
    async fn handle(
            &self,
            myself: ActorRef<Self::Msg>,
            message: Self::Msg,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        match message {
            WorkerMessage::FactoryPing(time) => {
                cast!(state.factory, FactoryMessage::WorkerPong(state.wid, time.elapsed()))?;
            }
            WorkerMessage::Dispatch(job) => {
                let rx = state.custom_start.clone();
                spawn_blocking(move || {
                    let file = File::open(job.msg)?;
                    let file_cache = ReadCache::new(file);
                    let obj = object::File::parse(&file_cache)?;

                    for symbol in obj.dynamic_symbols() {
                        let name = if let Some(name) = symbol.name().ok() { name } else { continue };
                        let name = Name::from(name);
                        let demangled = name.try_demangle(DemangleOptions::complete());
                        cast!(rx, Symbol { name: demangled.into(), offset: symbol.address() })?
                    }
                    
                    Ok::<_, ActorProcessingErr>(())
                }).await??;
               
                cast!(state.factory, FactoryMessage::Finished(state.wid, job.key))?;
            }
        }
        Ok(())
    }
}

pub struct OdexParserWorker;

impl Actor for OdexParserWorker {
    type Msg = WorkerMessage<(), PathBuf>;
    type State = WorkerStartContext<(), PathBuf, ActorRef<Symbol>>;
    type Arguments = WorkerStartContext<(), PathBuf, ActorRef<Symbol>>;
    
    async fn pre_start(
            &self,
            myself: ActorRef<Self::Msg>,
            args: Self::Arguments,
        ) -> Result<Self::State, ActorProcessingErr> {
        Ok(args)
    }
    
    async fn handle(
            &self,
            myself: ActorRef<Self::Msg>,
            message: Self::Msg,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        match message {
            WorkerMessage::FactoryPing(time) => {
                cast!(state.factory, FactoryMessage::WorkerPong(state.wid, time.elapsed()))?;
            }
            WorkerMessage::Dispatch(job) => {
                let mut symbols = oat_symbols(job.msg).await?;
                
                while let Some(symbol) = symbols.next().await {
                    cast!(state.custom_start, symbol)?;
                }
                
                cast!(state.factory, FactoryMessage::Finished(state.wid, job.key))?;
            }
        }
        Ok(())
    }
}

pub struct SoParserWorkerBuilder(ActorRef<Symbol>);

impl WorkerBuilder<SoParserWorker, ActorRef<Symbol>> for SoParserWorkerBuilder {
    fn build(&mut self, wid: ractor::factory::WorkerId) -> (SoParserWorker, ActorRef<Symbol>) {
        (SoParserWorker, self.0.clone())
    }
}

pub type SoParserFactory = Factory::<(), PathBuf, ActorRef<Symbol>, SoParserWorker, QueuerRouting<(), PathBuf>, DefaultQueue<(), PathBuf>>;

pub struct OdexParserWorkerBuilder(ActorRef<Symbol>);

impl WorkerBuilder<OdexParserWorker, ActorRef<Symbol>> for OdexParserWorkerBuilder {
    fn build(&mut self, wid: ractor::factory::WorkerId) -> (OdexParserWorker, ActorRef<Symbol>) {
        (OdexParserWorker, self.0.clone())
    }
}

pub type OdexParserFactory = Factory::<(), PathBuf, ActorRef<Symbol>, OdexParserWorker, QueuerRouting<(), PathBuf>, DefaultQueue<(), PathBuf>>;


pub async fn spawn_odex_parser_factory(actor: ActorRef<Symbol>) -> (ActorRef<FactoryMessage<(), PathBuf>>, JoinHandle<()>) {
    let factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(OdexParserWorkerBuilder(actor)))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(16)
        .build();
    
    let (factory, handle) = Actor::spawn(None, OdexParserFactory::default(), factory_args).await.unwrap();
    
    (factory, handle)
}

pub async fn spawn_so_parser_factory(actor: ActorRef<Symbol>) -> (ActorRef<FactoryMessage<(), PathBuf>>, JoinHandle<()>) {
    let factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(SoParserWorkerBuilder(actor)))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(16)
        .build();
    
    let (factory, handle) = Actor::spawn(None, SoParserFactory::default(), factory_args).await.unwrap();
    
    (factory, handle)
}

pub struct SymbolPathHandler;

impl Actor for SymbolPathHandler {
    type Arguments = (ActorRef<FactoryMessage<(), PathBuf>>, ActorRef<FactoryMessage<(), PathBuf>>);
    type State = (ActorRef<FactoryMessage<(), PathBuf>>, ActorRef<FactoryMessage<(), PathBuf>>);
    type Msg = SymbolFilePath;
    
    async fn pre_start(
            &self,
            myself: ActorRef<Self::Msg>,
            args: Self::Arguments,
        ) -> Result<Self::State, ActorProcessingErr> {
        Ok(args)
    }
    
    async fn handle(
            &self,
            myself: ActorRef<Self::Msg>,
            message: Self::Msg,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        match message {
            SymbolFilePath::Art(_) => Ok(()),
            SymbolFilePath::Oat(path) | SymbolFilePath::Odex(path) => Ok(cast!(state.0, FactoryMessage::Dispatch(Job {
                key: (),
                msg: path,
                options: JobOptions::default(),
                accepted: None,
            }))?),
            SymbolFilePath::So(path) => Ok(cast!(state.1, FactoryMessage::Dispatch(Job {
                key: (),
                msg: path,
                options: JobOptions::default(),
                accepted: None,
            }))?),
        }
    }
}

pub struct SymbolHandler;

impl Actor for SymbolHandler {
    type Arguments = Arc<tantivy::IndexWriter<TantivyDocument>>;
    type State = (Field, Field, Arc<tantivy::IndexWriter<TantivyDocument>>);
    type Msg = Symbol;
    
    async fn pre_start(
            &self,
            myself: ActorRef<Self::Msg>,
            args: Self::Arguments,
        ) -> Result<Self::State, ActorProcessingErr> {
        let symbol_name = args.index().schema().get_field("symbol_name").unwrap();
        let symbol_offset = args.index().schema().get_field("symbol_offset").unwrap();
        Ok((symbol_name, symbol_offset, args))
    }

    async fn handle(
            &self,
            myself: ActorRef<Self::Msg>,
            message: Self::Msg,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
        let (symbol_name, symbol_offset, writer) = state;

        writer.add_document(doc!(
            *symbol_name => message.name,
            *symbol_offset => message.offset,
        ))?;

        Ok(())
    }
}