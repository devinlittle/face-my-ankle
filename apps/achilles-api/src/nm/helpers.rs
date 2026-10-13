use std::collections::HashMap;

use anyhow::Result;
use zbus::{
    Connection,
    zvariant::{ObjectPath, OwnedObjectPath, Value},
};

use crate::{
    nm::{
        access_point_proxy::AccessPointProxy,
        active_proxy::ActiveProxy,
        device_proxy::DeviceProxy,
        helpers::WifiConnectionError::{ConnectionActivationFailed, DBusError, DoesNotExist},
        network_manager_proxy::NetworkManagerProxy,
        settings_connection_proxy::ConnectionProxy,
        settings_proxy::SettingsProxy,
        wireless_proxy::WirelessProxy,
    },
    structs::net::ApInfo,
};

pub async fn active_proxy<'a>(
    conn: &'a Connection,
    dev: &'a OwnedObjectPath,
) -> Result<ActiveProxy<'a>, zbus::Error> {
    ActiveProxy::builder(conn).path(dev)?.build().await
}

pub async fn nm_proxy<'a>(conn: &'a Connection) -> Result<NetworkManagerProxy<'a>, zbus::Error> {
    NetworkManagerProxy::builder(conn).build().await
}

pub async fn dev_proxy<'a>(
    conn: &'a Connection,
    dev: &'a OwnedObjectPath,
) -> Result<DeviceProxy<'a>, zbus::Error> {
    DeviceProxy::builder(conn).path(dev)?.build().await
}

pub async fn settings_proxy<'a>(
    conn: &'a Connection,
    dev: &'a OwnedObjectPath,
) -> Result<SettingsProxy<'a>, zbus::Error> {
    SettingsProxy::builder(conn).path(dev)?.build().await
}

pub async fn wireless_proxy<'a>(
    conn: &'a Connection,
    dev: &'a OwnedObjectPath,
) -> Result<WirelessProxy<'a>, zbus::Error> {
    WirelessProxy::builder(conn).path(dev)?.build().await
}

pub async fn access_point_proxy<'a>(
    conn: &'a Connection,
    dev: &'a OwnedObjectPath,
) -> Result<AccessPointProxy<'a>, zbus::Error> {
    AccessPointProxy::builder(conn).path(dev)?.build().await
}

pub async fn find_wifi_device(conn: &Connection) -> Result<OwnedObjectPath, zbus::Error> {
    let nm = NetworkManagerProxy::new(conn).await?;
    for path in nm.get_devices().await? {
        let dev = DeviceProxy::builder(conn).path(&path)?.build().await?;
        if dev.device_type().await? == 2 {
            return Ok(path);
        }
    }
    Err(zbus::Error::Failure("no WiFi device found".into()))
}

pub async fn read_ap(conn: &Connection, path: OwnedObjectPath) -> Result<ApInfo> {
    let ap = access_point_proxy(conn, &path).await?;
    let flags = ap.flags().await?;
    let wpa = ap.wpa_flags().await?;
    let rsn = ap.rsn_flags().await?;

    Ok(ApInfo {
        ssid: String::from_utf8_lossy(&ap.ssid().await?).into_owned(),
        strength: ap.strength().await?,
        frequency: ap.frequency().await?,
        bssid: ap.hw_address().await?,
        secured: (flags & 0x1) != 0 || wpa != 0 || rsn != 0,
        path,
    })
}

pub async fn list_networks(conn: &Connection, dev: &OwnedObjectPath) -> Result<Vec<ApInfo>> {
    let w = wireless_proxy(conn, dev).await?;
    let mut aps = Vec::new();
    for p in w.get_all_access_points().await? {
        if let Ok(ap) = read_ap(conn, p).await
            && !ap.ssid.is_empty()
        {
            aps.push(ap);
        }
    }

    //aps.sort_by(|a, b| b.strength.cmp(&a.strength));
    aps.sort_by_key(|a| a.strength);
    let mut seen = std::collections::HashSet::new();
    aps.retain(|a| seen.insert(a.ssid.clone()));
    Ok(aps)
}

async fn find_saved_connection(conn: &Connection, ssid: &str) -> Result<Option<OwnedObjectPath>> {
    let settings = SettingsProxy::new(conn).await?;
    for p in settings.list_connections().await? {
        let sc = ConnectionProxy::builder(conn).path(&p)?.build().await?;
        let s = sc.get_settings().await?;
        if let Some(w) = s.get("802-11-wireless")
            && let Some(v) = w.get("ssid")
            && let Ok(bytes) = <Vec<u8>>::try_from(v.clone())
            && bytes == ssid.as_bytes()
        {
            return Ok(Some(p));
        }
    }
    Ok(None)
}

#[derive(thiserror::Error, Debug)]
pub enum WifiConnectionError {
    #[error("SSID provided does not exist")]
    DoesNotExist,
    #[error("Incorrect Password")]
    PasswordWrong,
    #[error("Network Activation failed")]
    ConnectionActivationFailed,
    #[error("Internal D-Bus Error")]
    DBusError,
}

pub async fn connect(
    conn: &Connection,
    dev: &OwnedObjectPath,
    ssid: String,
    password: Option<String>,
) -> Result<OwnedObjectPath, WifiConnectionError> {
    let nm = nm_proxy(conn).await.map_err(|_| DBusError)?;
    let none = ObjectPath::try_from("/").map_err(|_| DoesNotExist)?;

    if let Some(saved) = find_saved_connection(conn, ssid.as_str())
        .await
        .map_err(|_| DoesNotExist)?
    {
        println!("{:?}", saved.clone().into_inner());
        return nm
            .activate_connection(&saved, dev, &none)
            .await
            .map_err(|_| ConnectionActivationFailed);
    }

    let ap = list_networks(conn, dev)
        .await
        .map_err(|_| DoesNotExist)?
        .into_iter()
        .find(|a| a.ssid == ssid)
        .ok_or(DoesNotExist)?;

    let mut s_conn: HashMap<&str, &Value> = HashMap::new();
    let wireless_type: Value = "802-11-wireless".into();
    let ssid_id: Value = ssid.clone().into();
    s_conn.insert("type", &wireless_type);
    s_conn.insert("id", &ssid_id);

    let mut s_wifi: HashMap<&str, &Value> = HashMap::new();
    let ssid_as_bytes = &Value::from(ssid.as_bytes().to_vec());
    let ssid_mode: Value = "infrastructure".into();
    s_wifi.insert("ssid", ssid_as_bytes);
    s_wifi.insert("mode", &ssid_mode);

    let mut settings: HashMap<&str, HashMap<&str, &Value>> = HashMap::new();
    settings.insert("connection", s_conn);
    settings.insert("802-11-wireless", s_wifi);

    let key_mgnt: Value = "wpa-psk".into();

    let pw = password.unwrap_or_default();
    let mut sec: HashMap<&str, &Value> = HashMap::new();
    let password_value: Value = pw.into();
    sec.insert("key-mgmt", &key_mgnt);
    sec.insert("psk", &password_value);
    settings.insert("802-11-wireless-security", sec);

    let (_profile, active) = nm
        .add_and_activate_connection(settings, dev, &ap.path)
        .await
        .map_err(|_| ConnectionActivationFailed)?;
    Ok(active)
}

pub async fn disconnect(conn: &Connection, dev: &OwnedObjectPath) -> Result<()> {
    dev_proxy(conn, dev)
        .await?
        .disconnect()
        .await
        .map_err(|_| anyhow::anyhow!("failed to disconnect device"))
}

pub async fn forget(conn: &Connection, ssid: &str) -> Result<bool> {
    if let Some(p) = find_saved_connection(conn, ssid).await? {
        ConnectionProxy::builder(conn)
            .path(&p)?
            .build()
            .await?
            .delete()
            .await?;
        return Ok(true);
    }
    Ok(false)
}

pub async fn set_wifi_enabled(conn: &Connection, on: bool) -> Result<()> {
    let _ = NetworkManagerProxy::new(conn)
        .await?
        .set_wireless_enabled(on)
        .await;

    Ok(())
}

pub async fn request_scan(conn: &Connection, dev: &OwnedObjectPath) -> Result<()> {
    let _ = wireless_proxy(conn, dev)
        .await?
        .request_scan(HashMap::new())
        .await;

    Ok(())
}

pub async fn get_ap_info(conn: &Connection, dev: &OwnedObjectPath, ssid: String) -> Result<ApInfo> {
    let w = wireless_proxy(conn, dev).await?;
    for p in w.get_all_access_points().await? {
        if let Ok(ap) = read_ap(conn, p).await
            && !ap.ssid.is_empty()
            && ap.ssid == ssid
        {
            return Ok(ap);
        }
    }

    Err(anyhow::anyhow!("AP Not Found"))
}

/* 
async fn list_saved_connections(conn: &Connection) -> Result<Option<Vec<ApInfo>>> {
    let settings = SettingsProxy::new(conn).await?;

    let ap: Vec<ApInfo> = Vec::new();

    for p in settings.list_connections().await? {
        let sc = ConnectionProxy::builder(conn).path(&p)?.build().await?;
    }
    Ok(None)
}
*/
