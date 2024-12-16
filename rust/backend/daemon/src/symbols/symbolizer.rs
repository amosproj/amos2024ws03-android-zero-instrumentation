use std::{fs::File, io, path::Path, sync::Arc};

use object::{Object, ObjectSymbol, ReadCache, ReadRef};
use ractor::{concurrency::oneshot, Actor};
use serde::{de, Deserialize, Deserializer};
use symbolic_common::Name;
use symbolic_demangle::{demangle, Demangle, DemangleOptions};
use tokio::{fs, process::Command, sync::{mpsc, oneshot}, task::spawn_blocking};
use tokio_process_stream::{Item, ProcessLineStream};
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use serde::de::Error;
use tracing::{error, warn};

use super::walking::SymbolFilePath;

#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub offset: u64
}

#[derive(Deserialize)]
struct OatSymbol {
    method: String,
    #[serde(deserialize_with = "u64_from_hex_str")]
    offset: u64,
}

impl Into<Symbol> for OatSymbol {
    fn into(self) -> Symbol {
        Symbol { name: self.method, offset: self.offset }
    }
}

fn u64_from_hex_str<'de, D>(de: D) -> Result<u64, D::Error> where D: Deserializer<'de> {
    let offset = String::deserialize(de)?;
    
    if !offset.starts_with("0x") {
        return Err(D::Error::custom("offset not starting with 0x"))
    }

    u64::from_str_radix(&offset[2..], 16).map_err(D::Error::custom)
}

fn log_map_item(item: Item<String>) -> Option<String> {
    match item {
        Item::Stdout(s) => Some(s),
        Item::Stderr(e) => {
            warn!("{}", e);
            None
        },
        Item::Done(Err(e)) => {
            error!("{}", e);
            None
        }
        Item::Done(Ok(_)) => {
            None
        }
    }
}

fn parse_oatdump_line(line: String) -> Option<Symbol> {
    match serde_json::from_str::<OatSymbol>(&line) {
        Ok(oat_symbol) => Some(oat_symbol.into()),
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}

fn filter_log_empty(symbol: &Symbol) -> bool {
    if symbol.offset == 0 {
        warn!("{} is not compiled", symbol.name);
        false
    } else {
        true
    }
}

async fn oatdata_offset<P: AsRef<Path>>(path: P) -> Result<u64, io::Error> {
    let path = path.as_ref().to_owned();
    match spawn_blocking(move || {
        let file = File::open(path)?;
        let file_cache = ReadCache::new(file);
        let obj = object::File::parse(&file_cache).map_err(io::Error::other)?;

        let section = obj
            .dynamic_symbols()
            .find(|s| s.name() == Ok("oatdata"))
            .ok_or(io::Error::other("oatdata not found"))?;

        Ok(section.address())
    }).await {
        Ok(v) => v,
        Err(e) => {
            Err(io::Error::other(e))
        },
    }
    
}

pub async fn oat_symbols<P: AsRef<Path>>(path: P) -> Result<impl Stream<Item = Symbol>, io::Error> {
    let path = path.as_ref();
    let oatdata_offset = oatdata_offset(path).await?;
    let mut cmd = Command::new("oatdump");
    cmd.arg(&format!("--oat-file={}", path.display()));
    cmd.arg("--dump-method-and-offset-as-json");
    cmd.arg("--no-disassemble");
    
    Ok(ProcessLineStream::try_from(cmd)?
        .filter_map(log_map_item)
        .filter_map(parse_oatdump_line)
        .filter(filter_log_empty)
        .map(move |symbol| Symbol { name: symbol.name, offset: symbol.offset + oatdata_offset }))
}

pub fn so_symbols<P: AsRef<Path>>(path: P) -> impl Stream<Item = Symbol> {
    let path = path.as_ref().to_owned();

    let (tx, rx) = mpsc::channel(128);

    spawn_blocking(move || {
        let file = File::open(path)?;
        let file_cache = ReadCache::new(file);
        let obj = object::File::parse(&file_cache).map_err(io::Error::other)?;
        
        for symbol in obj.dynamic_symbols() {
            let name = if let Some(name) = symbol.name().ok() { name } else { continue };
            let name = Name::from(name);
            let demangled = name.try_demangle(DemangleOptions::complete());
            tx.blocking_send(Symbol { name: demangled.into(), offset: symbol.address() }).map_err(io::Error::other)?;
        }

        Ok::<(), io::Error>(())
    });

    ReceiverStream::new(rx)
}
