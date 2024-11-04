use std::{thread, time::{self, SystemTime, UNIX_EPOCH}};

use shared::ziofa::{LoadEbpfProgramResponse, ProgramResponse1, ProgramResponse2};
use tokio::sync::mpsc::Sender;
use tonic::Status;

pub async fn ebpf_program1(tx: &Sender<Result<LoadEbpfProgramResponse, Status>>) {
    for i in 1..10 {
        // get current millis
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // send packet to client
        tx.send(Ok(LoadEbpfProgramResponse {
            pr1: Some(ProgramResponse1 {time: u64::try_from(time).unwrap()}),
            pr2: None
        }))
        .await
        .unwrap();

        // sleep one second
        let one_second = time::Duration::from_secs(1);
        thread::sleep(one_second);
    }
}

pub async fn ebpf_program2(tx: &Sender<Result<LoadEbpfProgramResponse, Status>>) {
    for i in 1..10 {
        // get current millis
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // send packet to client
        tx.send(Ok(LoadEbpfProgramResponse {
            pr1: None,
            pr2: Some(ProgramResponse2 {time: time.to_string()})
        }))
        .await
        .unwrap();

        // sleep one second
        let one_second = time::Duration::from_secs(1);
        thread::sleep(one_second);
    }
}
