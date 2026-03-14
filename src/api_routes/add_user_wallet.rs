use axum::{Json, extract::Extension, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Deserialize)]
pub struct AddWallet {
    pub wallets: Vec<String>,
}

#[derive(Serialize)]
pub struct AddWalletReturn {
    success: bool,
    msg: String,
}

pub enum AddWalletReturnStatus {
    Success(AddWalletReturn), // The string passed will be the success message
    Fail(AddWalletReturn),
}

impl IntoResponse for AddWalletReturnStatus {
    fn into_response(self) -> axum::response::Response {
        match self {
            AddWalletReturnStatus::Success(success) => {
                (StatusCode::OK, Json(success)).into_response()
            }
            AddWalletReturnStatus::Fail(fail) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(fail)).into_response()
            }
        }
    }
}

pub async fn add_wallet_logic(
    Extension(state): Extension<UnboundedSender<(Vec<String>, u8)>>,
    Json(data): Json<AddWallet>,
) -> AddWalletReturnStatus {
    match state.send((data.wallets, 0)) {
        Ok(_) => println!("Successfully sent wallet"),
        Err(err) => {
            println!("error sensing wallet:: {err:#?}");
            return AddWalletReturnStatus::Fail(AddWalletReturn {
                msg: "Unable to complete the operation".to_string(),
                success: false,
            });
        }
    }
    AddWalletReturnStatus::Success(AddWalletReturn {
        msg: "Successfully added the wallet".to_string(),
        success: true,
    })
}
