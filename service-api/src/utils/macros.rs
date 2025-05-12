#[macro_export]
macro_rules! require_field {
    ($field:expr) => {
        if $field.is_none() {
            return
            Err((
                    axum::http::StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({
                        "message": format!("Missing required field: {}", stringify!($field)),
                    })).into_response(),
            ))

        }
    };
}
