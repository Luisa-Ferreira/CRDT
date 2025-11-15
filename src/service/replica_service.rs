use tonic::{Request, Response, Status};
use crate::replica::replica_service_server::{ReplicaService, ReplicaServiceServer};
use crate::replica::{StateRequest, StateResponse};
use crate::replica::Replica;
use crate::crdt_set::RWSet;
use tonic::transport::{ClientTlsConfig, Channel};
use tonic::tls::Identity;

pub struct MyReplicaService;

#[tonic::async_trait]
impl ReplicaService for MyReplicaService {
    async fn send_state(&mut self, request: Request<StateRequest>) -> Result<Response<StateResponse>, Status> {
        let request = request.into_inner();
        println!("Recebido estado de: {}", request.from);

        self.rset.merge(&received_state);

        Ok(Response::new(StateResponse { success: true }))
    }

    fn mtls_configuration() -> ClientTlsConfig {
        let cert = std::fs::read("certs/server-cert.pem").unwrap();
        let key = std::fs::read("certs/server-key.pem").unwrap();

        let identity = Identity::from_pem(cert, key);

        ClientTlsConfig::new().identity(identity)
    }
    

}
