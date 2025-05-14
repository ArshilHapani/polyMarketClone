#[macro_export]
macro_rules! require_field {
    ($field:expr) => {
        if $field.is_none() {
            let full = stringify!($field);
            let short = full.split('.').last().unwrap_or(full);
            return
            Err((
                    axum::http::StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({
                        "message": format!("Missing required field: {}", short),
                    })).into_response(),
            ))
        }
    };
}

#[macro_export]
macro_rules! require_fields_raw_response {
    ($field:expr) => {
        if $field.is_none() {
            let full = stringify!($field);
            let short = full.split('.').last().unwrap_or(full);
            return Err((
                    axum::http::StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({
                        "message": format!("Missing required field: {}", short),
                    })),
            ))
        }
    };
}
