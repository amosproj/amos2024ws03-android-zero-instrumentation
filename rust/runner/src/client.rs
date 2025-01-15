// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use runner::{Execution, HostSpec, Log};

use std::{collections::VecDeque, fs::File, io::{stdin, BufRead, BufReader, Read, Write}, path::PathBuf, sync::mpsc::{Receiver, SyncSender}, thread::spawn};

use adb_client::{ADBDeviceExt, ADBServer};

struct ChannelReader {
    rx: Receiver<Vec<u8>>,
    buf: VecDeque<u8>,
    closed: bool,
}

impl ChannelReader {
    fn new(rx: Receiver<Vec<u8>>) -> Self {
        Self {
            rx,
            buf: VecDeque::new(),
            closed: false,
        }
    }
}

#[derive(Clone)]
struct ChannelWriter {
    tx: SyncSender<Vec<u8>>,
}

impl ChannelWriter {
    fn new(tx: SyncSender<Vec<u8>>) -> Self {
        Self { tx }
    }
}

impl Read for ChannelReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if self.closed || !self.buf.is_empty() {
                return self.buf.read(buf);
            }
            
            match self.rx.recv() {
                Ok(data) => {
                    self.buf.extend(data);
                }
                Err(_) => {
                    self.closed = true;
                }
            }
        }
    }
}


impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.tx.send(buf.to_vec()).map_err(|_| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "channel closed"))?;
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn rw_pair() -> (ChannelWriter, ChannelReader) {
    let (tx, rx) = std::sync::mpsc::sync_channel(0);
    (ChannelWriter::new(tx), ChannelReader::new(rx))
}

pub fn main() {
    let mut args = std::env::args();
    
    let _ = args.next().expect("first arguments is current executable");
    
    let host_spec = {
        let arg = args.next().expect("required argument");
        serde_json::from_str::<HostSpec>(&arg).expect("invalid json")
    };
    
    let exe_src_path = PathBuf::from(args.next().expect("required argument"));
    
    let exe_name = exe_src_path
        .file_name()
        .expect("no file name")
        .to_str()
        .expect("file name is not valid utf-8");

    let exe_dest_path = format!("/data/local/tmp/{}", exe_name);
    
    let mut server = ADBServer::default();
    let mut device = server.get_device().expect("cannot get device");
    
    let source_file = File::open(&exe_src_path).expect("cannot open source file");
    device.push(source_file, &exe_dest_path).expect("cannot push file");
    
    let runner = File::open(host_spec.runner_path).expect("cannot open runner");
    device.push(runner, "/data/local/tmp/runner").expect("cannot push runner");
    
    let (output_writer, output_reader) = rw_pair();
    let (mut command_writer, mut command_reader) = rw_pair();

    let (command_tx, command_rx) = std::sync::mpsc::sync_channel(0);

    ctrlc::set_handler({
        let command_tx = command_tx.clone();
        move || {
            let _ = command_tx.send("\"Exit\"".to_owned());
        }
    }).expect("failed to set ctrlc handler");
    
    spawn({
        let command_tx = command_tx.clone();
        move || {
            let _ = stdin().lines().next();
            let _ = command_tx.send("\"Exit\"".to_owned());
        }
    });
    
    let shell_process = spawn(move || {
        device.shell(&mut command_reader, Box::new(output_writer))
            .map_err(std::io::Error::other)?;
        
        Ok::<(), std::io::Error>(())
    });
    
    let printer_process = spawn (move || {
        for line in BufReader::new(output_reader).lines().skip(1) {
            let line = line?;
            match serde_json::from_str::<Log>(&line) {
                Ok(Log::Stderr(l)) => eprintln!("{}", l),
                Ok(Log::Stdout(l)) => println!("{}", l),
                Ok(Log::InternalError(l)) => panic!("{}", l),
                Err(e) => eprintln!("Error parsing log line: {}, original line: {}", e, line)
            };
        }
        Ok::<(), std::io::Error>(())
    });
    
    let cmd_process = spawn(move || {
        for cmd in command_rx {
            if cmd.is_empty() {
                break;
            }
            writeln!(command_writer, "{}", cmd)?;
        }
        Ok::<(), std::io::Error>(())
    });
    
    let execution = Execution {
        command: exe_dest_path.to_string(),
        args: host_spec.args,
        env: host_spec.env,
        root: host_spec.root
    };
    
    command_tx.send(format!("/data/local/tmp/runner '{}' && exit", serde_json::to_string(&execution).unwrap())).expect("failed to send command");
    
    printer_process.join().expect("thread did not panic").expect("failed to join printer");
    
    command_tx.send("".to_owned()).expect("failed to send command");

    cmd_process.join().expect("thread did not panic").expect("failed to join command handler");
    shell_process.join().expect("thread did not panic").expect("failed to join shell handler");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_pair() {
        let (mut writer, reader) = rw_pair();
        
        let read = std::thread::spawn(move || {
            let mut lines = BufReader::new(reader).lines();
            assert!(matches!(lines.next(), Some(Ok(line)) if line == "echo hello"));
            assert!(matches!(lines.next(), Some(Ok(line)) if line == "echo world"));
            assert!(lines.next().is_none());
        });
        
        let write = std::thread::spawn(move || {
            writeln!(writer, "echo hello").unwrap();
            writeln!(writer, "echo world").unwrap();
        });
        
        write.join().unwrap();
        read.join().unwrap();
    }
}

