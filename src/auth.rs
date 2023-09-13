use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

pub async fn get_codepair(
    client_id: String,
) -> Result<models::CodePairResponse, errors::AuthorizationError> {
    let req = json!(
        {
            "client_id": client_id,
            "response_type": "device_code",
            "scope": "profile"
        }
    );

    let res = HTTP_CLIENT
        .post("https://api.amazon.com/auth/o2/create/codepair")
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded;charset=UTF-8",
        )
        .form(&req)
        .send()
        .await?;

    let body = res.json::<serde_json::Value>().await?;

    if let Ok(err_res) = serde_json::from_value::<models::ErrorResponse>(body.clone()) {
        return Err(errors::AuthorizationError::from_error_response(&err_res));
    }

    if let Ok(code_pair) = serde_json::from_value::<models::CodePairResponse>(body.clone()) {
        return Ok(code_pair);
    }

    panic!("Unknown response: {:?}", body.to_string());
}

pub async fn perform_code_exchange(
    user_code: String,
    device_code: String,
) -> Result<models::TokenResponse, errors::AuthorizationError> {
    let req = json!(
        {
            "grant_type": "device_code",
            "device_code": device_code,
            "user_code": user_code,
        }
    );

    let res = HTTP_CLIENT
        .post("https://api.amazon.com/auth/o2/token")
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded;charset=UTF-8",
        )
        .form(&req)
        .send()
        .await?;

    let body = res.json::<serde_json::Value>().await?;

    if let Ok(err_res) = serde_json::from_value::<models::ErrorResponse>(body.clone()) {
        return Err(errors::AuthorizationError::from_error_response(&err_res));
    }

    if let Ok(token_res) = serde_json::from_value::<models::TokenResponse>(body.clone()) {
        return Ok(token_res);
    }

    panic!("Unknown response: {:?}", body.to_string());
}

pub async fn perform_token_refresh(
    client_id: String,
    refresh_token: String,
) -> Result<models::TokenResponse, errors::AuthorizationError> {
    let req = json!(
        {
            "grant_type": "refresh_token",
            "refresh_token": refresh_token,
            "client_id": client_id,
        }
    );

    let res = HTTP_CLIENT
        .post("https://api.amazon.com/auth/o2/token")
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded;charset=UTF-8",
        )
        .form(&req)
        .send()
        .await?;

    let body = res.json::<serde_json::Value>().await?;

    if let Ok(err_res) = serde_json::from_value::<models::ErrorResponse>(body.clone()) {
        return Err(errors::AuthorizationError::from_error_response(&err_res));
    }

    if let Ok(token_res) = serde_json::from_value::<models::TokenResponse>(body.clone()) {
        return Ok(token_res);
    }

    panic!("Unknown response: {:?}", body.to_string());
}

pub mod errors {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum AuthorizationError {
        #[error("The request is missing a required parameter, has an invalid value, or is otherwise improperly formed.")]
        InvalidRequest,

        #[error("The client is not authorized to request an authorization code.")]
        UnauthorizedClient,

        #[error("The resource owner or authorization server denied this request.")]
        AccessDenied,

        #[error("The request specified an unsupported response type.")]
        UnsupportedResponseType,

        #[error("The client requested the wrong scope.")]
        InvalidScope,

        #[error("The authorization server encountered an unexpected error")]
        ServerError,

        #[error("The authorization server is currently unavailable due to a temporary overload or scheduled maintenance.")]
        TemporarilyUnavailable,

        #[error("The user has not yet entered their user code at the verification URL")]
        AuthorizationPending,

        #[error("The device is polling too quickly. Make Device Token Requests only as frequently as indicated by the interval in the Device Authorization Response")]
        SlowDown,

        #[error(
            "The device_code has expired. You will need to make a new Device Authorization Request"
        )]
        ExpiredToken,

        #[error("Failed to make the request.")]
        FetchError(#[from] reqwest::Error),

        #[error("Failed to parse the response")]
        ParseError(#[from] serde_json::Error),
    }

    impl AuthorizationError {
        pub fn from_error_response(res: &super::models::ErrorResponse) -> Self {
            match res.error.as_str() {
                "invalid_request" => AuthorizationError::InvalidRequest,
                "unauthorized_client" => AuthorizationError::UnauthorizedClient,
                "access_denied" => AuthorizationError::AccessDenied,
                "unsupported_response_type" => AuthorizationError::UnsupportedResponseType,
                "invalid_scope" => AuthorizationError::InvalidScope,
                "server_error" => AuthorizationError::ServerError,
                "temporarily_unavailable" => AuthorizationError::TemporarilyUnavailable,
                "authorization_pending" => AuthorizationError::AuthorizationPending,
                "slow_down" => AuthorizationError::SlowDown,
                "expired_token" => AuthorizationError::ExpiredToken,
                _ => panic!("Unknown error type: {}", res.error),
            }
        }
    }
}

pub mod models {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ErrorResponse {
        pub error: String,
        pub error_description: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CodePairResponse {
        pub user_code: String,
        pub device_code: String,
        pub verification_uri: String,
        pub expires_in: i64,
        pub interval: u64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TokenResponse {
        pub access_token: String,
        pub refresh_token: String,
        pub token_type: String,
        pub expires_in: u64,
    }
}
