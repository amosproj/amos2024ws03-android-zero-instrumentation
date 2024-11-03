mod ebpf_info;

use ebpf_info::EbpfInfo;
use tonic::{transport::Server, Request, Response, Status};
use tokio;

fn start_server(){

}

struct MapFunc{
    do_map: fn()
}

fn get_from_prog_map(_msg: String) -> (EbpfInfo, MapFunc){
    todo!(" not yet impl");
}

#[tokio::main]
async fn main() {
    // start server that accepts msg. from frontend

    //WTF??? why unwrap
    let addr = "[::1]:50051".to_socket_addrs().unwrap().next().unwrap();

    let service = SomeService();

    //
    let mut server = Server::builder()
            .add_service(service)
            .serve(addr);

    // what if unauthorized user connects to server
    loop{
        // wait till user connects to server
        server.await;

        let (socket, msg) = wait_for_msg_from_frontend(server_socket);

        // find ebpf prog
        let (ebpf_info, map_to_api) = get_from_prog_map(msg);

        //load ebpf prog.

        ebpf_info.load();


        //execute function talks to eBPF prog.
        map_to_api(socket);

        //unload ebpf
        unload_ebpf_prog()
    }
}

fn wait_for_msg_from_frontend(server_socket) ->  {
    todo!()
}
