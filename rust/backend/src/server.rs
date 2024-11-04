use tonic::transport::Server;
use crate::main_helpers;

pub async fn server_forever() {
    let sock_addr = main_helpers::get_socket_addr();
    let server_impl = ZoifaImpl::new();
    let service = ZoifaServer::new(server_impl);
    Server::builder()
        .add_service(service)
        .serve(sock_addr).await.unwrap();
}