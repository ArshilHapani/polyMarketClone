use async_nats::{
    connect,
    jetstream::{self, Context},
};

pub struct AppState {
    pub jetstream: Context,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        let nats_url = std::env::var("NC_URL")
            .map_err(|_| "NC_URL environment variable not set".to_string())?;

        let nc = connect(nats_url).await?;
        let js = jetstream::new(nc);

        Ok(AppState { jetstream: js })
    }
}
