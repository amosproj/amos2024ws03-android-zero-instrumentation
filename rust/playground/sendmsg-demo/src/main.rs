// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use std::{
    io::{IoSlice, IoSliceMut}, os::fd::{AsRawFd, OwnedFd}, process::id, sync::mpsc, thread::{sleep, spawn}, time::Duration
};

use console::style;
use dialoguer::{theme::ColorfulTheme, Input};
use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle};
use nix::sys::socket::{
        getsockopt, recvmsg, sendmsg, setsockopt, socketpair,
        sockopt::SndBuf,
        AddressFamily, MsgFlags, SockFlag, SockType,
    };

pub struct Inputs<'a> {
    packet_amount: Input<'a, u64>,
    time_between_packets: Input<'a, u64>,
    time_to_block: Input<'a, u64>,
}

impl<'a> Inputs<'a> {
    pub fn new(theme: &'a ColorfulTheme) -> Self {
        let packet_amount = Input::<u64>::with_theme(theme)
            .with_prompt("How many packets do you want to send?".to_owned());

        let time_between_packets = Input::<u64>::with_theme(theme)
            .with_prompt("How long to wait between packets in seconds?".to_owned());

        let time_to_block = Input::<u64>::with_theme(theme)
            .with_prompt("How long to block when sending packets in seconds?".to_owned());

        Self {
            packet_amount,
            time_between_packets,
            time_to_block,
        }
    }

    pub fn interact(self) -> Result<Data, dialoguer::Error> {
        let packet_amount = self.packet_amount.interact()?;
        let time_between_packets = self.time_between_packets.interact()?;
        let time_to_block = self.time_to_block.interact()?;

        Ok(Data {
            packet_amount,
            time_between_packets,
            time_to_block,
        })
    }
}

pub struct Data {
    packet_amount: u64,
    time_between_packets: u64,
    time_to_block: u64,
}

#[derive(Clone)]
pub struct Bars {
    _multi: MultiProgress,
    packets_sent: ProgressBar,
    packets_received: ProgressBar,
    wait_bar: ProgressBar,
    block_bar: ProgressBar,
}

impl Bars {

    #[allow(clippy::literal_string_with_formatting_args)]
    pub fn new(packet_amount: u64, time_between_packets: u64, time_to_block: u64) -> Self {
        let multi = MultiProgress::new();

        let send_style = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.green} {pos:>7}/{len:7} {msg}")
            .expect("template should be valid");

        let receive_style = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.blue} {pos:>7}/{len:7} {msg}")
            .expect("template should be valid");
        
        let sleep_style = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.yellow} {pos:>7}/{len:7} {msg}")
            .expect("template should be valid");
        
        let blocking_style = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.red} {pos:>7}/{len:7} {msg}")
            .expect("template should be valid");
        

        let packets_sent = ProgressBar::new(packet_amount)
            .with_style(send_style)
            .with_message("packets sent")
            .with_finish(ProgressFinish::WithMessage("done".into()));

        let packets_received = ProgressBar::new(packet_amount)
            .with_style(receive_style)
            .with_message("packets received")
            .with_finish(ProgressFinish::WithMessage("done".into()));

        let wait_bar = ProgressBar::new(time_between_packets * 50)
            .with_style(sleep_style)
            .with_message("sleeping")
            .with_finish(ProgressFinish::WithMessage("done".into()));

        let block_bar = ProgressBar::new(time_to_block * 50)
            .with_style(blocking_style)
            .with_message("blocking")
            .with_finish(ProgressFinish::WithMessage("done".into()));

        multi.add(wait_bar.clone());
        multi.add(block_bar.clone());
        multi.add(packets_sent.clone());
        multi.add(packets_received.clone());

        Self {
            _multi: multi,
            packets_sent,
            packets_received,
            wait_bar,
            block_bar,
        }
    }

    pub fn tick(&self) {
        self.wait_bar.tick();
        self.block_bar.tick();
        self.packets_sent.tick();
        self.packets_received.tick();
    }
}

struct Sender {
    fd: OwnedFd,
    blocking_buffer: Vec<u8>,
}

impl Sender {
    pub fn new(fd: OwnedFd, blocking_size: usize) -> Self {
        Self {
            fd,
            blocking_buffer: vec![1u8; blocking_size],
        }
    }

    pub fn send(&self) -> Result<(), nix::Error> {
        sendmsg::<()>(
            self.fd.as_raw_fd(),
            &[IoSlice::new(&self.blocking_buffer)],
            &[],
            MsgFlags::empty(),
            None,
        )?;
        Ok(())
    }
}

struct Receiver {
    fd: OwnedFd,
    blocking_buffer: Vec<u8>,
}

impl Receiver {
    pub fn new(fd: OwnedFd, blocking_size: usize) -> Self {
        Self {
            fd,
            blocking_buffer: vec![0u8; blocking_size],
        }
    }

    pub fn recv(&mut self) -> Result<(), nix::Error> {
        recvmsg::<()>(
            self.fd.as_raw_fd(),
            &mut [IoSliceMut::new(&mut self.blocking_buffer)],
            None,
            MsgFlags::empty(),
        )?;
        recvmsg::<()>(
            self.fd.as_raw_fd(),
            &mut [IoSliceMut::new(&mut self.blocking_buffer)],
            None,
            MsgFlags::empty(),
        )?;

        Ok(())
    }
}

fn create_blocking_pair() -> Result<(Sender, Receiver), nix::Error> {
    let (tx, rx) = socketpair(
        AddressFamily::Unix,
        SockType::Stream,
        None,
        SockFlag::empty(),
    )?;

    // set send buffer to minimal size
    setsockopt(&tx, SndBuf, &0)?;

    let actual_sndbuf_size = getsockopt(&tx, SndBuf)?;

    Ok((
        Sender::new(tx, actual_sndbuf_size),
        Receiver::new(rx, actual_sndbuf_size),
    ))
}

struct SenderTask {
    sender: Sender,
    packet_amount: u64,
    time_between_packets: u64,
    packets_sent: ProgressBar,
    wait_bar: ProgressBar,
    wait_finished_tx: mpsc::Sender<()>,
    receiver_finished_rx: mpsc::Receiver<()>,
}

impl SenderTask {
    pub fn execute(self) -> Result<(), nix::Error> {
        for _ in 0..self.packet_amount {
            self.wait_bar.reset();
            self.wait_bar.set_message("sleeping");
            for _ in 0..(self.time_between_packets * 50) {
                sleep(Duration::from_millis(20));
                self.wait_bar.inc(1);
            }
            self.wait_bar.finish_using_style();
            self.wait_finished_tx.send(()).unwrap();

            self.packets_sent.inc(1);

            self.sender.send()?;
        }
        self.packets_sent.finish_using_style();
        self.receiver_finished_rx.recv().unwrap();

        Ok(())
    }
}

struct ReceiverTask {
    receiver: Receiver,
    packet_amount: u64,
    time_to_block: u64,
    packets_received: ProgressBar,
    block_bar: ProgressBar,
    wait_finished_rx: mpsc::Receiver<()>,
    receiver_finished_tx: mpsc::Sender<()>,
}

impl ReceiverTask {
    pub fn execute(mut self) -> Result<(), nix::Error> {
        for _ in 0..self.packet_amount {
            self.wait_finished_rx.recv().unwrap();

            self.block_bar.reset();
            self.block_bar.set_message("blocking");
            for _ in 0..(self.time_to_block * 50) {
                sleep(Duration::from_millis(20));
                self.block_bar.inc(1);
            }
            self.block_bar.finish_using_style();

            self.receiver.recv()?;

            self.packets_received.inc(1);
        }

        self.packets_received.finish_using_style();
        self.receiver_finished_tx.send(()).unwrap();

        Ok(())
    }
}

fn create_task_pair(
    packet_amount: u64,
    time_between_packets: u64,
    time_to_block: u64,
    bars: Bars,
) -> Result<(SenderTask, ReceiverTask), nix::Error> {
    let (sender, receiver) = create_blocking_pair()?;
    let (wait_finished_tx, wait_finished_rx) = mpsc::channel::<()>();
    let (receiver_finished_tx, receiver_finished_rx) = mpsc::channel::<()>();

    let sender = SenderTask {
        sender,
        packet_amount,
        time_between_packets,
        packets_sent: bars.packets_sent.clone(),
        wait_bar: bars.wait_bar.clone(),
        wait_finished_tx,
        receiver_finished_rx,
    };

    let receiver = ReceiverTask {
        receiver,
        packet_amount,
        time_to_block,
        packets_received: bars.packets_received.clone(),
        block_bar: bars.block_bar.clone(),
        wait_finished_rx,
        receiver_finished_tx,
    };

    Ok((sender, receiver))
}

fn main() -> anyhow::Result<()> {
    println!("{} {}\n", style("Process PID:").bold(), style(id().to_string()).bold().green());

    let Data {
        packet_amount,
        time_between_packets,
        time_to_block,
        ..
    } = Inputs::new(&ColorfulTheme::default()).interact()?;
    
    println!();

    let bars = Bars::new(packet_amount, time_between_packets, time_to_block);

    bars.tick();

    let (sender, receiver) =
        create_task_pair(packet_amount, time_between_packets, time_to_block, bars)?;

    let receiver_handle = spawn(move || receiver.execute());

    sender.execute()?;
    receiver_handle.join().unwrap()?;

    Ok(())
}
