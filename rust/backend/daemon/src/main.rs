// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{borrow::BorrowMut, hash::BuildHasherDefault, sync::{Arc, RwLock}, time::Duration};

use clap::builder;
use constants::INDEX_PATH;
use futures::{stream::FuturesUnordered, StreamExt};
use object::write;
use ractor::{call, Actor, ActorStatus};
use symbols::{actors::{SymbolActor, SymbolActorMsg}, index::index};
use tantivy::{aggregation::bucket, collector::{Count, DocSetCollector, TopDocs}, directory::MmapDirectory, doc, query::{AllQuery, PhraseQuery, Query, QueryParser}, schema::{Facet, FacetOptions, IndexRecordOption, SchemaBuilder, TextFieldIndexing, TextOptions, FAST, INDEXED, STORED, TEXT}, store::{Compressor, ZstdCompressor}, tokenizer::{LowerCaser, SimpleTokenizer, TextAnalyzer, TextAnalyzerBuilder, Token, TokenFilter, TokenStream, Tokenizer, TokenizerManager}, Index, IndexSettings, Searcher, TantivyDocument, Term};
use tokio::{fs, runtime::{Builder, Runtime}, spawn, sync::mpsc, task::{spawn_blocking, JoinSet}, time::interval};
use tokio_stream::{wrappers::ReceiverStream};
use tracing_subscriber::EnvFilter;
mod constants;
mod ebpf_utils;
mod helpers;
mod procfs_utils;
mod server;
mod features;
mod collector;
mod symbols;
mod registry;
mod filesystem;

async fn create_index() {
    fs::remove_dir_all(INDEX_PATH).await.unwrap();
    
    let symbol_actor = SymbolActor::spawn().await.unwrap();
    call!(symbol_actor, SymbolActorMsg::ReIndex).unwrap();
    symbol_actor.stop(None);
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();
    
    create_index().await;
    
    let index = index().unwrap();
    
    let symbol_name = index.schema().get_field("symbol_name").unwrap();
    
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    let query = QueryParser::for_index(&index, vec![symbol_name]).parse_query(r#"*"#).unwrap();
    
    let count = searcher.search(&query, &Count).unwrap();
    println!("{count}");
    
    
    
    /* 

    // apparently needed...
    helpers::bump_rlimit();

    server::serve_forever().await;
    */
}
