use axum::{Extension, Json, extract::State, response::IntoResponse};
use hyper::StatusCode;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::{
    nm::helpers::{
        WifiConnectionError::{DoesNotExist, PasswordWrong},
        connect, disconnect, list_networks, request_scan, set_wifi_enabled,
    },
    routes::AppState,
    structs::{
        AuthenticatedUser,
        net::{AccessPoint, SetWifiStatusInput, WifiConnectInput},
    },
};

#[utoipa::path(
    patch,
    path = "/net/wifi/enabled",
    request_body = SetWifiStatusInput,
    responses(
        (status = 204, description = "Changed the status of the wifi interface"),
        (status = 500, description = "Unable to change wifi interface status")
    ),
    tag = "net"
)]
pub async fn set_wifi_status(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(req): Json<SetWifiStatusInput>,
) -> Result<impl IntoResponse, StatusCode> {
    set_wifi_enabled(&state.zbus_con.0, req.enabled)
        .await
        .map_err(|err| {
            error!("Error changing wlan int: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("WiFi enabled is {}", req.enabled);
    Ok(())
}

#[utoipa::path(
    post,
    path = "/net/wifi/scan",
    responses(
        (status = 202, description = "Scan intiated"),
        (status = 500, description = "Unable to scan")
    ),
    tag = "net"
)]
pub async fn scan_wifi(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
) -> Result<impl IntoResponse, StatusCode> {
    request_scan(&state.zbus_con.0, &state.zbus_con.1)
        .await
        .map_err(|err| {
            error!("Error initaiating scan: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    info!("Scanning for WiFi networks");
    Ok(())
}

#[utoipa::path(
    get,
    path = "/net/wifi/networks",
    responses(
        (status = 200, description = "Scan intiated", body = Vec<AccessPoint>),
        (status = 500, description = "Unable to scan")
    ),
    tag = "net"
)]
pub async fn get_networks(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let output: Vec<AccessPoint> = list_networks(&state.zbus_con.0, &state.zbus_con.1)
        .await
        .map_err(|err| {
            error!("Error retrieving networks: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .iter()
        .map(AccessPoint::from)
        .collect();

    info!("Returned a list of near by WiFi networks");
    Ok(Json(output).into_response())
}

#[utoipa::path(
    get,
    path = "/net/wifi/connect",
    request_body = WifiConnectInput,
    responses(
        (status = 204, description = "Connected"),
        (status = 404, description = "SSID doesnt exist"),
        (status = 401, description = "incorrect credentials"),
        (status = 500, description = "Unable to connect")
    ),
    tag = "net"
)]
pub async fn connect_to_network(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(req): Json<WifiConnectInput>,
) -> Result<impl IntoResponse, StatusCode> {
    connect(&state.zbus_con.0, &state.zbus_con.1, req.ssid, req.password)
        .await
        .map_err(|err| match err {
            DoesNotExist => StatusCode::NOT_FOUND,
            PasswordWrong => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    info!("Connected to WiFi");
    Ok(().into_response())
}

#[utoipa::path(
    delete,
    path = "/net/wifi/disconnect",
    responses(
        (status = 200, description = "Disconnected successfully"),
        (status = 500, description = "Unable to scan")
    ),
    tag = "net"
)]
pub async fn disconnect_from_network(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
) -> Result<impl IntoResponse, StatusCode> {
    disconnect(&state.zbus_con.0, &state.zbus_con.1)
        .await
        .map_err(|_| {
            warn!("failed to disconnect from network");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("disconnected from WiFi");
    Ok(().into_response())
}

#[utoipa::path(
    delete,
    path = "/net/wifi/{ssid}",
    responses(
        (status = 200, description = "Successfully forgot network", body = Vec<AccessPoint>),
        (status = 500, description = "Unable to scan")
    ),
    tag = "net"
)]
pub async fn forget_network(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
) -> Result<impl IntoResponse, StatusCode> {
    disconnect(&state.zbus_con.0, &state.zbus_con.1)
        .await
        .map_err(|_| {
            warn!("failed to disconnect from network");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("disconnected from WiFi");
    Ok(().into_response())
}
