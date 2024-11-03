mod ebpf_info;
mod main_helpers;

use std::collections::HashMap;
use ebpf_info::EbpfInfo;
use std::net::ToSocketAddrs;
use tokio;
use tonic::transport::Server;

const PROG_MAP: HashMap<String, (EbpfInfo, fn(Server))> = HashMap::from([
    ("Example_prog", )
]);


fn get_from_prog_map(_msg: String) -> (EbpfInfo, fn(Server)){
    todo!(" not yet impl");
    // prog_map:
    // Explain_string, EbpfInfo, to_api_function
}

fn get_prog_descr_s() -> std::array{
    let length_of_prog_map = PROG_MAP.len();
    let m_arr: std::array = PROG_MAP.iter().map(|x| {x[0]}).unwrap();

}

#[tokio::main]
async fn main() {
    env_logger::init();

    // apparently needed...
    main_helpers::bump_rlimit();

    //
    // start server
    //
    let addr = "[::1]:50051".to_socket_addrs().unwrap().next().unwrap();

    // One server-socket? One connection to frontend?
    let mut server = Server::builder()
            .add_service(SomeService())
            .serve(addr);

    //TODO: what if unauthorized service connects to server?
    loop{

        //
        // wait till user connects to server
        //
        let sock = server.await.unwrap();

        let prog_id = wait_for_msg_from_frontend(&sock);

        //
        // find ebpf prog
        //
        let (ebpf_info, map_to_api) = get_from_prog_map(msg);

        //load ebpf prog.
        ebpf_info.load();

        //execute function that talks to eBPF prog.
        map_to_api(sock);

        //unload ebpf
        unload_ebpf_prog()
    }
}

fn wait_for_msg_from_frontend(server_socket) ->  {
    todo!()
}
