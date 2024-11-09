use serde::{Deserialize, Serialize};
use shared::config::{Configuration as ProtoConfig, EbpfEntry as ProtoEbpfEntry};
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter};

#[derive(Serialize, Deserialize)]
pub(crate) struct Configuration {
    entries: Vec<EbpfEntry>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct EbpfEntry {
    hr_name: String,
    description: String,
    ebpf_name: String,
    fn_id: u64,
}

impl From<Configuration> for ProtoConfig {
    fn from(config: Configuration) -> Self {
        ProtoConfig {
            entries: config
                .entries
                .into_iter()
                .map(ProtoEbpfEntry::from)
                .collect(),
        }
    }
}

impl From<ProtoConfig> for Configuration {
    fn from(proto: ProtoConfig) -> Self {
        Configuration {
            entries: proto.entries.into_iter().map(EbpfEntry::from).collect(),
        }
    }
}

impl From<EbpfEntry> for ProtoEbpfEntry {
    fn from(entry: EbpfEntry) -> Self {
        ProtoEbpfEntry {
            hr_name: entry.hr_name,
            description: entry.description,
            ebpf_name: entry.ebpf_name,
            fn_id: entry.fn_id,
        }
    }
}

impl From<ProtoEbpfEntry> for EbpfEntry {
    fn from(proto: ProtoEbpfEntry) -> Self {
        EbpfEntry {
            hr_name: proto.hr_name,
            description: proto.description,
            ebpf_name: proto.ebpf_name,
            fn_id: proto.fn_id,
        }
    }
}

impl Configuration {
    pub fn load_from_file(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &str) -> io::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &self)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Self::Error> {
        //TODO: Implement this function
        Ok(())
    }
}
