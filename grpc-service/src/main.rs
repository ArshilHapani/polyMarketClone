pub mod procedures;
pub mod state;

pub mod markets_tonic {
    include!("generated/markets.rs");
}
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{Any, CorsLayer};

const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("generated/descriptor.bin");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reflector_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .include_reflection_service(true)
        .build_v1alpha()?;

    println!("GRPC server running on port http://localhost:5010");
    let addr = "0.0.0.0:5010".parse()?;

    Server::builder()
        .accept_http1(true)
        .layer(GrpcWebLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .add_service(reflector_service)
        .serve(addr)
        .await?;

    Ok(())
}
