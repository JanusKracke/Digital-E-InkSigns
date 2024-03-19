use embedded_svc::wifi::ClientConfiguration;
use esp_idf_svc::wifi::AuthMethod;
use heapless::String;
use std::str::FromStr;

pub fn get_wifi() -> ClientConfiguration {
    ClientConfiguration {
        ssid: String::from_str("SSID").unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: String::from_str("PASSWORD").unwrap(),
        channel: None,
    }
}

pub fn get_server() -> std::string::String {
    "http://SERVER:PORT/".to_string()
}