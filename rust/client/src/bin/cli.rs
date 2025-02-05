// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::{
    ffi::CStr, path::Path, process::id, sync::LazyLock
};

use anyhow::bail;
use clap::{Parser, Subcommand};
use client::Client;
use rusqlite::{
    ffi::{SQLITE_OPEN_CREATE, SQLITE_OPEN_READWRITE, SQLITE_OPEN_WAL},
    params, Connection, OpenFlags,
};
use shared::{
    config::{
        BlockingConfig, Configuration, Filter, GarbageCollectConfig, MissingBehavior, StringFilter, UInt32Filter,
        WriteConfig,
    },
    events::{
        event::EventData, log_event::LogEventData, write_event::WriteSource, Event, EventContext, GarbageCollectEvent, LogEvent, WriteEvent
    }, google::protobuf::{Duration, Timestamp},
};
use tokio::{fs, select, signal::ctrl_c};
use tokio_stream::StreamExt;

pub type Result<T> = anyhow::Result<T>;

#[derive(Debug, Parser)]
struct Args {
    /// Address the client binds to
    #[arg(long, default_value = "http://[::1]:50051")]
    addr: String,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Sendmsg {
        /// Pid for which to track sendmsg calls
        #[arg(short, long)]
        pid: u32,
    },

    /// Get the current config
    GetConfig,

    /// Set the default config
    SetConfig,

    /// List all running processes
    Processes {
        /// Only output number of processe
        #[arg(short, long)]
        silent: bool,
    },

    /// Create an Index for all Symbols on the System
    IndexSymbols,

    /// Search via query for symbols
    SearchSymbols {
        /// The query string
        query: String,

        /// The limit of symbols sent by the server
        limit: u64,
    },

    /// Finds the symbol given symbol name and library path
    GetSymbolOffset {
        /// The name of the symbol
        symbol_name: String,

        /// The path of the library containing the symbol
        library_path: String,
    },

    /// Collects everything
    Collect {
        /// The path to the sqlite database
        sqlite_path: String,
    },
}

async fn sendmsg(client: &mut Client, pid: u32) -> Result<()> {
    client
        .set_configuration(Configuration {
            blocking_config: Some(BlockingConfig {
                filter: Some(Filter {
                    pid_filter: Some(UInt32Filter {
                        r#match: vec![pid],
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                threshold: Some(32_000_000),
            }),
            ..Default::default()
        })
        .await?;

    let mut stream = client.init_stream().await?;

    while let Some(next) = stream.next().await {
        println!("{next:?}");
    }

    Ok(())
}

async fn set_config(client: &mut Client) -> Result<()> {
    client.set_configuration(Configuration::default()).await?;
    println!("Success");
    Ok(())
}

async fn list_processes(client: &mut Client, silent: bool) -> Result<()> {
    let processes = client.list_processes().await?;
    if silent {
        println!("Number of processes: {}", processes.len());
        return Ok(());
    }

    for (i, p) in processes.iter().enumerate() {
        println!("Process {}: {:?}", i, p);
    }

    Ok(())
}

async fn index_symbols(client: &mut Client) -> Result<()> {
    println!("Indexing Symbols, this can take a while...");
    client.index_symbols().await?;
    println!("SUCCESS");
    Ok(())
}

async fn search_symbols(client: &mut Client, query: String, limit: u64) -> Result<()> {
    let symbols = client.search_symbols(query, limit).await?;

    let mut count = 0;
    for symbol in symbols {
        println!(
            "method: {} | offset: {} | library_path: {}",
            symbol.method, symbol.offset, symbol.path
        );
        count += 1;
    }
    println!("Total number of symbols: {count}");

    Ok(())
}

async fn get_symbol_offset(
    client: &mut Client,
    symbol_name: String,
    library_path: String,
) -> Result<()> {
    let offset = client.get_symbol_offset(symbol_name, library_path).await?;

    if let Some(offset) = offset {
        println!("Found offset: {offset}");
    } else {
        println!("Did not find symbol");
    }

    Ok(())
}

const EVENT_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS events (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        pid INTEGER NOT NULL,
        tid INTEGER NOT NULL,
        comm TEXT NOT NULL,
        cmdline TEXT NOT NULL,
        timestamp INTEGER NOT NULL
    )
";

const WRITE_EVENT_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS write_events (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        file_descriptor INTEGER NOT NULL,
        bytes_written INTEGER NOT NULL,
        file_path TEXT NOT NULL,
        source TEXT NOT NULL,
        event INTEGER NOT NULL,
        FOREIGN KEY(event) REFERENCES events(id)
    )
";

const GARBAGE_COLLECT_EVENT_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS garbage_collect_events (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        target_footprint INTEGER NOT NULL,
        num_bytes_allocated INTEGER NOT NULL,
        gcs_completed INTEGER NOT NULL,
        gc_cause INTEGER NOT NULL,
        duration_ns INTEGER NOT NULL,
        freed_objects INTEGER NOT NULL,
        freed_bytes INTEGER NOT NULL,
        freed_los_objects INTEGER NOT NULL,
        freed_los_bytes INTEGER NOT NULL,
        event INTEGER NOT NULL,
        FOREIGN KEY(event) REFERENCES events(id)
    )
";

const INSERT_INTO_EVENTS: &str = "
    INSERT INTO events (
        pid, tid, comm, cmdline, timestamp
    ) VALUES (
        ?1, ?2, ?3, ?4, ?5
    )
";

const INSERT_INTO_WRITE_EVENTS: &str = "
    INSERT INTO write_events (
        file_descriptor, bytes_written, file_path, source, event
    ) VALUES (
        ?1, ?2, ?3, ?4, ?5
    )
";

const INSERT_INTO_GARBAGE_COLLECT_EVENTS: &str = "
    INSERT INTO garbage_collect_events (
        target_footprint, num_bytes_allocated, gcs_completed, gc_cause, duration_ns, freed_bytes, freed_objects, freed_los_bytes, freed_los_objects, event
    ) VALUES (
        ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10
    )
";

const LAST_ROW_QUERY: &str = "
    SELECT LAST_INSERT_ROWID()
";

static ALL_MATCHING_FILTER: LazyLock<Filter> = LazyLock::new(|| Filter {
    pid_filter: Some(UInt32Filter {
        missing_behavior: MissingBehavior::Match.into(),
        not_match: vec![id()],
        ..Default::default()
    }),
    comm_filter: Some(StringFilter {
        missing_behavior: MissingBehavior::Match.into(),
        ..Default::default()
    }),
    cmdline_filter: Some(StringFilter {
        missing_behavior: MissingBehavior::Match.into(),
        ..Default::default()
    }),
    exe_path_filter: Some(StringFilter {
        missing_behavior: MissingBehavior::Match.into(),
        ..Default::default()
    }),
});

struct DestructuredEventContent {
    pub pid: u32,
    pub tid: u32,
    pub timestamp: u64,
    pub comm: String,
    pub cmdline: String,
    pub data: LogEventData,
}

impl DestructuredEventContent {
    pub async fn new(event: Event) -> Result<Self> {
        let Some(EventData::Log(LogEvent { context: Some(EventContext { pid, tid, timestamp }), log_event_data: Some(data) })) = event.event_data else { bail!("no data") };
        
        let comm = get_comm(pid).await.unwrap_or_default();
        let cmdline = get_cmdline(pid).await.unwrap_or_default();
        let timestamp = timestamp.as_nanos();

        Ok(Self {
            pid,
            tid,
            timestamp,
            comm,
            cmdline,
            data
        })
    }
    
    pub fn insert(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            INSERT_INTO_EVENTS,
             params![self.pid, self.tid, self.comm, self.cmdline, self.timestamp]
        )?;
        let event_id = conn.query_row(
            LAST_ROW_QUERY,
             (),
              |row| row.get::<_, usize>(0)
        )?;
        
        match &self.data {
            LogEventData::Write(write_event) => {
                let source = match write_event.source() {
                    WriteSource::Undefined => "UNDEFINED",
                    WriteSource::Write64 => "WRITE64",
                    WriteSource::Writev => "WRITEV",
                    WriteSource::Writev2 => "WRITEV2",
                    WriteSource::Write => "WRITE",
                };
                let WriteEvent {
                    file_descriptor,
                    bytes_written,
                    file_path,
                    ..
                } = write_event;
                conn.execute(
                    INSERT_INTO_WRITE_EVENTS,
                    params![file_descriptor, bytes_written, file_path, source, event_id]
                )?;
            },
            LogEventData::GarbageCollect(garbage_collect_event) => {
                let GarbageCollectEvent {
                    target_footprint,
                    num_bytes_allocated,
                    gcs_completed,
                    gc_cause,
                    duration_ns,
                    freed_bytes,
                    freed_objects,
                    freed_los_bytes,
                    freed_los_objects,
                    ..
                } = garbage_collect_event;
                let freed_objects = *freed_objects as i64;
                let freed_los_objects = *freed_los_objects as i64;

                conn.execute(
                    INSERT_INTO_GARBAGE_COLLECT_EVENTS,
                    params![target_footprint, num_bytes_allocated, gcs_completed, gc_cause, duration_ns, freed_bytes, freed_objects, freed_los_bytes, freed_los_objects, event_id]
                )?;
                
            },
            _ => {}
        }
        
        Ok(())
    }
}

trait AsNanos {
    fn as_nanos(&self) -> u64;
}

impl AsNanos for Option<Timestamp> {
    fn as_nanos(&self) -> u64 {
        self.map(|t| {
            (t.seconds as u64 * 1_000_000) + t.nanos as u64
        }).unwrap_or_default()
    }
}

impl AsNanos for Option<Duration> {
    fn as_nanos(&self) -> u64 {
        self.map(|t| {
            (t.seconds as u64 * 1_000_000) + t.nanos as u64
        }).unwrap_or_default()
    }
}

async fn collect<P: AsRef<Path>>(client: &mut Client, path: P) -> Result<()> {
    client
        .set_configuration(Configuration {
            garbage_collect_config: Some(GarbageCollectConfig {
                filter: Some(ALL_MATCHING_FILTER.clone()),
            }),
            write_config: Some(WriteConfig {
                filter: Some(ALL_MATCHING_FILTER.clone()),
            }),
            ..Default::default()
        })
        .await?;
    
    let conn = Connection::open_with_flags(
        path.as_ref(),
        OpenFlags::from_bits_truncate(SQLITE_OPEN_CREATE | SQLITE_OPEN_READWRITE | SQLITE_OPEN_WAL),
    )?; 
    
    conn.execute(EVENT_TABLE, ())?;
    conn.execute(WRITE_EVENT_TABLE, ())?;
    conn.execute(GARBAGE_COLLECT_EVENT_TABLE, ())?;

    let mut stream = client.init_stream().await?;

    let ctrlc = ctrl_c();
    tokio::pin!(ctrlc);

    loop {
        select! {
            _ = &mut ctrlc => {
                break;
            }
            event = stream.next() => {
                let Some(event) = event else { break };
                let Ok(event) = event else { continue };
                
                let event = DestructuredEventContent::new(event).await?;
                event.insert(&conn)?;
            }
        }
    }

    conn.close().unwrap();

    Ok(())
}

async fn get_comm(pid: u32) -> Result<String> {
    let comm_bytes = fs::read(&format!("/proc/{pid}/comm")).await?;
    Ok(String::from_utf8_lossy(&comm_bytes[..comm_bytes.len().saturating_sub(1)]).into_owned())
}

async fn get_cmdline(pid: u32) -> Result<String> {
    let cmdline_bytes = fs::read(&format!("/proc/{pid}/cmdline")).await?;
    if let Ok(first_part) = CStr::from_bytes_until_nul(&cmdline_bytes[..]) {
        Ok(first_part.to_string_lossy().into_owned())
    } else {
        Ok(String::from_utf8_lossy(&cmdline_bytes).into_owned())
    }
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    println!("Trying to connect to {}", args.addr);
    let mut client = Client::connect(args.addr.to_owned()).await?;

    match args.cmd {
        Commands::Sendmsg { pid } => {
            sendmsg(&mut client, pid).await?;
        }
        Commands::GetConfig => {
            println!("{:?}", client.get_configuration().await?);
        }
        Commands::SetConfig => {
            set_config(&mut client).await?;
        }
        Commands::Processes { silent } => {
            list_processes(&mut client, silent).await?;
        }
        Commands::IndexSymbols => {
            index_symbols(&mut client).await?;
        }
        Commands::SearchSymbols { query, limit } => {
            search_symbols(&mut client, query, limit).await?;
        }
        Commands::GetSymbolOffset {
            symbol_name,
            library_path,
        } => {
            get_symbol_offset(&mut client, symbol_name, library_path).await?;
        }
        Commands::Collect { sqlite_path } => {
            collect(&mut client, sqlite_path).await?;
        }
    }

    Ok(())
}
