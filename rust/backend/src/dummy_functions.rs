use std::{thread, time::{self, SystemTime, UNIX_EPOCH}};

use shared::ziofa::{ConcreteEbpfStreamObject1, ConcreteEbpfStreamObject2, EbpfStreamObject};
use tokio::sync::mpsc::Sender;
use tonic::Status;

pub async fn ebpf_program1(tx: Sender<Result<EbpfStreamObject, Status>>) {
    for _ in 1..10 {
        // get current millis
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let obj = ConcreteEbpfStreamObject1 {
                    time: time as u64
                };

        tx.send(Ok(obj.create_stream_object()))
        .await
        .unwrap();

        // sleep one second
        let one_second = time::Duration::from_secs(1);
        thread::sleep(one_second);
    }
}

pub async fn ebpf_program2(tx: Sender<Result<EbpfStreamObject, Status>>) {
    for _ in 1..10 {
        // get current millis
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let obj = ConcreteEbpfStreamObject2{
                    time: time.to_string()
                };

        // send packet to client
        tx.send(Ok(obj.create_stream_object()))
        .await
        .unwrap();

        // sleep one second
        let one_second = time::Duration::from_secs(1);
        thread::sleep(one_second);
    }
}