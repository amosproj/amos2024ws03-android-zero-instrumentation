// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{io::{stdin, BufRead, BufReader}, process::{Child, Stdio}, sync::mpsc::Sender, thread::{spawn, JoinHandle}};

use rustix::process::getuid;

use runner::{Execution, Log};

fn spawn_self_as_root(current_exe: String, mut execution: Execution) {
    execution.root = false;
    
    let mut cmd = std::process::Command::new("su");
    cmd.arg("root");
    cmd.arg(current_exe);
    cmd.arg(serde_json::to_string(&execution).unwrap());
    cmd.spawn()
        .expect("failed to spawn")
        .wait()
        .expect("failed to wait");
}

fn spawn_execution(execution: Execution) -> Child {
    let mut cmd = std::process::Command::new(&execution.command);
    cmd.current_dir("/data/local/tmp");
    cmd.args(&execution.args);
    cmd.env_clear();
    for (key, value) in execution.env {
        cmd.env(key, value);
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.stdin(Stdio::piped());
    
    cmd.spawn().expect("failed to spawn")
}

fn spawn_writer() -> (Sender<Log>, JoinHandle<()>) {
    let (tx, rx) = std::sync::mpsc::channel();
    
    let writer = spawn(move || {
        rx.iter()
            .map(|log| serde_json::to_string(&log))
            .filter_map(Result::ok)
            .for_each(|log| println!("{log}"));
    });
    
    (tx, writer)
}

fn spawn_iterator(tx: Sender<Log>, iter: impl Iterator<Item = Log> + Send + 'static) -> JoinHandle<()> {
    spawn(move || {
        iter.for_each(|log| tx.send(log).expect("failed to send log"));
    })
}

fn spawn_shutdown(mut child: Child) -> (Sender<()>, JoinHandle<()>) {
    let (tx, rx) = std::sync::mpsc::channel();
    
    let shutdown = spawn(move || {
        rx.recv().expect("failed to receive shutdown signal");
        child.kill().expect("failed to kill child");
    });
    
    (tx, shutdown)
}

fn spawn_command_handler(tx: Sender<()>) -> JoinHandle<()> {
    spawn(move || {
        let _ = stdin().lines()
            .map_while(Result::ok)
            .map(|line| serde_json::from_str::<runner::Command>(&line))
            .filter_map(Result::ok)
            .next();
        
        tx.send(()).expect("failed to send shutdown signal");
    })
}

pub fn main() {
    let mut args = std::env::args();
    
    let current_exe = args.next().expect("first arguments is current executable");
    
    let execution = {
        let arg = args.next().expect("required argument");
        serde_json::from_str::<Execution>(&arg).expect("invalid json")
    };
    
    if execution.root && !getuid().is_root() {
        return spawn_self_as_root(current_exe, execution);
    }
    
    let mut child = spawn_execution(execution);
    
    let stdout = BufReader::new(child.stdout.take().unwrap());
    let stderr = BufReader::new(child.stderr.take().unwrap());
    
    let (tx, writer_handle) = spawn_writer();
    let stdout_handle = spawn_iterator(tx.clone(), stdout.lines().map(|line| match line {
        Ok(line) => Log::Stdout(line),
        Err(err) => Log::InternalError(err.to_string())
    }));
    let stderr_handle = spawn_iterator(tx, stderr.lines().map(|line| match line {
        Ok(line) => Log::Stderr(line),
        Err(err) => Log::InternalError(err.to_string())
    }));
    let (shutdown_tx, shutdown_handle) = spawn_shutdown(child);
    
    let _ = spawn_command_handler(shutdown_tx.clone());
    
    stdout_handle.join().expect("failed to join stdout handler");
    stderr_handle.join().expect("failed to join stderr handler");
    
    let _ = shutdown_tx.send(());
    
    writer_handle.join().expect("failed to join writer");
    shutdown_handle.join().expect("failed to join shutdown handler");
}