use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use hyper::StatusCode;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::{
    nm::helpers::{
        WifiConnectionError::{DoesNotExist, PasswordWrong},
        connect, disconnect, forget, list_networks, request_scan, set_wifi_enabled,
    },
    routes::AppState,
    structs::net::{AccessPoint, SetWifiStatusInput, WifiConnectInput},
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
        (status = 500, description = "Unable to get networks")
    ),
    tag = "net"
)]
pub async fn get_networks(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut output: Vec<AccessPoint> = list_networks(&state.zbus_con.0, &state.zbus_con.1)
        .await
        .map_err(|err| {
            error!("Error retrieving networks: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .iter()
        .map(AccessPoint::from)
        .collect();

    // Having networks displaued in order of strongest
    output.reverse();

    info!("Returned a list of near by WiFi networks");
    Ok(Json(output).into_response())
}

#[utoipa::path(
    post,
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
        (status = 500, description = "Unable to disconnect from network")
    ),
    tag = "net"
)]
pub async fn disconnect_from_network(
    State(state): State<Arc<AppState>>,
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
    params(
        ("ssid", description = "SSID of network to forget")
    ),
    responses(
        (status = 200, description = "Successfully forgot network"),
        (status = 500, description = "Unable to forget network")
    ),
    tag = "net"
)]
pub async fn forget_network(
    State(state): State<Arc<AppState>>,
    Path(ssid): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    forget(&state.zbus_con.0, ssid.as_str())
        .await
        .map_err(|_| {
            warn!("failed to forget network");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Forgotton WiFi Network");
    Ok(().into_response())
}

#[utoipa::path(
    get,
    path = "/net/wifi/{ssid}",
    params(
        ("ssid", description = "SSID of network to forget")
    ),
    responses(
        (status = 200, description = "Returns AP into", body = AccessPoint),
        (status = 404, description = "AP not found")
    ),
    tag = "net"
)]
pub async fn get_ap_info(
    State(state): State<Arc<AppState>>,
    Path(ssid): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let ap = crate::nm::helpers::get_ap_info(&state.zbus_con.0, &state.zbus_con.1, ssid)
        .await
        .map_err(|_| {
            warn!("failed to get ap information");
            StatusCode::NOT_FOUND
        })?;

    let ap: AccessPoint = AccessPoint::from(&ap);

    info!("Forgotton WiFi Network");
    Ok(Json(ap).into_response())
}

#[utoipa::path(
    get,
    path = "/net/int/ip",
    responses(
        (status = 200, description = "Returns Private IP", body = Option<String>),
    ),
    tag = "net"
)]
pub async fn get_ip(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Option<String>>, StatusCode> {
    let status = state.device_status.borrow().clone();
    Ok(Json(status.ip))
}

/*
#[utoipa::path(
    get,
    path = "/net/wifi/saved_connctions",
    responses(
        (status = 200, description = "Returns AP into", body = Vec<AccessPoint>),
        (status = 500, description = "Unable to get info")
    ),
    tag = "net"
)]
pub async fn list_saved_connections(
    State(state): State<Arc<AppState>>,
    Path(ssid): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Forgotton WiFi Network");
    Ok(Json("").into_response())
}
*/
