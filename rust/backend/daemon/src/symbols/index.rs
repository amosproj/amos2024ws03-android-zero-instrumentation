// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use std::io;

use tantivy::{
    directory::MmapDirectory,
    schema::{
        IndexRecordOption, SchemaBuilder, TextFieldIndexing, TextOptions, FAST, INDEXED, STORED,
        STRING,
    },
    store::{Compressor, ZstdCompressor},
    tokenizer::{
        LowerCaser, SimpleTokenizer, TextAnalyzer, Token, TokenFilter, TokenStream, Tokenizer,
        TokenizerManager,
    },
    Index, IndexSettings,
};

use crate::constants::INDEX_PATH;

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

impl<T: TokenStream> SplitCamelCaseTokenStream<'_, T> {
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

impl<T: TokenStream> TokenStream for SplitCamelCaseTokenStream<'_, T> {
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
        self.parts
            .last_mut()
            .unwrap_or_else(|| self.tail.token_mut())
    }
}

pub fn index() -> Result<Index, io::Error> {
    std::fs::create_dir_all(INDEX_PATH)?;

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
                        .set_index_option(IndexRecordOption::WithFreqsAndPositions),
                )
                .set_stored()
                .set_fast(Some("raw")),
        );
        builder.add_text_field("symbol_name_exact", STRING | STORED);
        builder.add_u64_field("symbol_offset", STORED | FAST | INDEXED);
        builder.add_text_field("library_path", STRING | STORED);
        builder.build()
    };

    let directory = MmapDirectory::open(INDEX_PATH).map_err(io::Error::other)?;

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
        .open_or_create(directory)
        .map_err(io::Error::other)?;

    Ok(index)
}
