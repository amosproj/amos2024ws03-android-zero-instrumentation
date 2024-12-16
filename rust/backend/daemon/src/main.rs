// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{borrow::BorrowMut, hash::BuildHasherDefault, sync::Arc};

use clap::builder;
use futures::stream::FuturesUnordered;
use object::write;
use symbols::{symbolizer::{oat_symbols, so_symbols}, walking::{self, all_symbol_files, SymbolFilePath}};
use tantivy::{aggregation::bucket, collector::{DocSetCollector, TopDocs}, directory::MmapDirectory, doc, query::{AllQuery, PhraseQuery, Query, QueryParser}, schema::{Facet, FacetOptions, IndexRecordOption, SchemaBuilder, TextFieldIndexing, TextOptions, FAST, INDEXED, STORED, TEXT}, store::{Compressor, ZstdCompressor}, tokenizer::{LowerCaser, SimpleTokenizer, TextAnalyzer, TextAnalyzerBuilder, Token, TokenFilter, TokenStream, Tokenizer, TokenizerManager}, Index, IndexSettings, Searcher, TantivyDocument, Term};
use tokio::{fs, spawn, task::spawn_blocking};
use tokio_stream::StreamExt;
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

struct SplitCamelCase;

#[derive(Clone)]
struct SplitCamelCaseFilter<T> {
    inner: T,
    cuts: Vec<usize>,
    parts: Vec<Token>,
}

struct SplitCamelCaseTokenStream<'a, T> {
    tail: T,
    cuts: &'a mut Vec<usize>,
    parts: &'a mut Vec<Token>,
}


impl TokenFilter for SplitCamelCase {
    type Tokenizer<T: Tokenizer> = SplitCamelCaseFilter<T>;
    
    fn transform<T: Tokenizer>(self, tokenizer: T) -> Self::Tokenizer<T> {
        SplitCamelCaseFilter {
            inner: tokenizer,
            cuts: Vec::new(),
            parts: Vec::new(),
        }
    }
}

impl<T: Tokenizer> Tokenizer for SplitCamelCaseFilter<T> {
    type TokenStream<'a> = SplitCamelCaseTokenStream<'a, T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        self.cuts.clear();
        self.parts.clear();
        SplitCamelCaseTokenStream {
            tail: self.inner.token_stream(text),
            cuts: &mut self.cuts,
            parts: &mut self.parts,
        }
    }
}

impl<'a, T: TokenStream> SplitCamelCaseTokenStream<'a, T> {
    fn split(&mut self) {
        let token = self.tail.token();
        let mut text = token.text.as_str();

        self.cuts.clear();
        for (index, char) in text.char_indices() {
            if char.is_uppercase() {
                self.cuts.push(index);
            }
        }
        
        for index in self.cuts.iter().rev() {
            let (head, tail) = text.split_at(*index);

            text = head;
            self.parts.push(Token {
                text: tail.to_owned(),
                ..*token
            });
        }
        
        self.parts.push(token.clone());
    }
}

impl<'a, T: TokenStream> TokenStream for SplitCamelCaseTokenStream<'a, T> {
    fn advance(&mut self) -> bool {
        self.parts.pop();
        
        if !self.parts.is_empty() {
            return true;
        }
        
        if !self.tail.advance() {
            return false;
        }
        
        self.split();
        true
    }

    fn token(&self) -> &tantivy::tokenizer::Token {
        self.parts.last().unwrap_or_else(|| self.tail.token())
    }

    fn token_mut(&mut self) -> &mut tantivy::tokenizer::Token {
        self.parts.last_mut().unwrap_or_else(|| self.tail.token_mut())
    }
}

fn index() -> Index {
    let code_tok = TextAnalyzer::builder(SimpleTokenizer::default())
        .filter(SplitCamelCase)
        .filter(LowerCaser)
        .build();
    
    let schema = {
        let mut builder = SchemaBuilder::new();
        builder.add_text_field(
            "symbol_name", 
            TextOptions::default()
                .set_indexing_options(
                    TextFieldIndexing::default()
                        .set_tokenizer("code")
                        .set_index_option(IndexRecordOption::WithFreqsAndPositions)
                    )
                .set_stored()
                .set_fast(Some("raw"))
                );
        builder.add_u64_field("symbol_offset", STORED | FAST | INDEXED);
        builder.build()
    };

    let index = Index::builder()
        .tokenizers({
            let tok = TokenizerManager::default();
            tok.register("code", code_tok);
            tok
        })
        .settings(IndexSettings {
            docstore_compression: Compressor::Zstd(ZstdCompressor::default()),
            ..IndexSettings::default()
        })
        .schema(schema)
        .open_or_create(MmapDirectory::open(INDEX_PATH).unwrap()).unwrap();
    
    index
}

const INDEX_PATH: &str = "/data/local/tmp/index";

async fn create_index() {
    fs::remove_dir_all(INDEX_PATH).await.unwrap();
    fs::create_dir_all(INDEX_PATH).await.unwrap();
    
    let index = index();
    
    let symbol_name = index.schema().get_field("symbol_name").unwrap();
    let symbol_offset = index.schema().get_field("symbol_offset").unwrap();
    
    let writer = Arc::new(index.writer(50_000_000).unwrap());
    
    let mut all_symbols = all_symbol_files();

    let mut tasks: FuturesUnordered<tokio::task::JoinHandle<()>> = FuturesUnordered::new();
    while let Some(path) = all_symbols.next().await {
        let writer = writer.clone();
        if tasks.len() == 4 {
            tasks.next().await.unwrap().unwrap();
        }
        match path {
            SymbolFilePath::Odex(path) | SymbolFilePath::Oat(path) => {
                tasks.push(spawn(async move {
                    println!("indexing {path:?}");
                    let mut oat_symbols = oat_symbols(path).await.unwrap();
                    while let Some(symbol) = oat_symbols.next().await {
                        let writer = writer.clone();
                        spawn_blocking(move || {
                            writer.add_document(doc!(
                                symbol_name => symbol.name,
                                symbol_offset => symbol.offset,
                            )).unwrap();
                        }).await.unwrap();
                    }
                }));
            }
            SymbolFilePath::So(path) =>  {
                tasks.push(spawn(async move {
                    println!("indexing {path:?}");
                    let mut so_symbols = so_symbols(path);
                    while let Some(symbol) = so_symbols.next().await {
                        let writer = writer.clone();
                        spawn_blocking(move || {
                            writer.add_document(doc!(
                                symbol_name => symbol.name,
                                symbol_offset => symbol.offset,
                            )).unwrap();
                        }).await.unwrap();
                    }
                }));
            }
            _ => {},
        }
    }
    while let Some(x) = tasks.next().await {
        x.unwrap();
    }
    let mut writer = Arc::into_inner(writer).unwrap();
    writer.commit().unwrap();
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();
    
    //create_index().await;
    
    let index = index();
    
    let symbol_name = index.schema().get_field("symbol_name").unwrap();
    
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    let query = QueryParser::for_index(&index, vec![symbol_name]).parse_query(r#"*"#).unwrap();
    
    for address in searcher.search(&query, &TopDocs::with_limit(10)).unwrap() {
        let doc: TantivyDocument = searcher.doc(address.1).unwrap();
        //println!("{:?}", query.explain(&searcher, address.1).unwrap());
        println!("{doc:?}");
    }
    
    
    /* 

    // apparently needed...
    helpers::bump_rlimit();

    server::serve_forever().await;
    */
}
