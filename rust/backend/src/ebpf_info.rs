use std::path::Path;
use aya::{Ebpf};

pub(crate) struct EbpfInfo {
    ebpf_prog: Path,
    proc_id: u32,
}

impl EbpfInfo {
    pub(crate) fn load(self){
        let prog = Ebpf::load_file(self.ebpf_prog);



    }
}