use axum::http::StatusCode;
use axum_extra::protobuf::Protobuf;
use service_api::markets::Market;

pub async fn get_markets() -> Result<Protobuf<Market>, StatusCode> {
    let resp = Protobuf(Market {
        id: "1".to_string(),
    });
    Ok(resp)
}
