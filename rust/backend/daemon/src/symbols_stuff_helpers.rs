use std::path::PathBuf;
use tokio::process::Command;
use std::fs::File;
use object::{Object, ObjectSymbol, ReadCache};
use std::io::Error;
use crate::constants::OATDUMP_PATH;
use crate::symbols_stuff::SymbolError;

pub async fn generate_json_oatdump(path: &PathBuf) -> Result<(), SymbolError> {
    let _oatdump_status = Command::new("oatdump")
        .args(vec![
            format!("--output={}", OATDUMP_PATH).as_str(),
            "--dump-method-and-offset-as-json",
            format!("--oat-file={}", path.to_str().unwrap().to_string()).as_str(),
        ])
        .spawn()?
        .wait()
        .await?;
    // TODO: Check for status [robin]
    //       do we even need the status -> if yes for what? [benedikt]
    Ok(())
}

pub async fn get_section_address(oat_path: &PathBuf) -> Result<u64, Error> {
    tokio::task::spawn_blocking({
        let path = oat_path.clone();
        move || get_symbol_address_from_oat(&path, "oatdata")
    })
    .await?
}

fn get_symbol_address_from_oat(path: &PathBuf, symbol_name: &str) -> Result<u64, Error> {
    let file = File::open(path)?;
    let file_cache = ReadCache::new(file);
    let obj = object::File::parse(&file_cache).unwrap();

    let section = obj
        .dynamic_symbols()
        .find(|s| s.name() == Ok(symbol_name))
        .unwrap();

    Ok(section.address())
}