use esp_idf_hal::modem::Modem;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_idf_svc::eventloop::{EspEventLoop,System};
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use embedded_svc::wifi::Configuration;
use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use embedded_svc::http::client::Client;

use crate::wifi_credentials;


pub fn setup_wifi(modem: Modem, sysloop: EspEventLoop<System>, nvs: EspNvsPartition<NvsDefault>) -> anyhow::Result<BlockingWifi<EspWifi<'static>>> {

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sysloop.clone(), Some(nvs))?,
        sysloop,
    )?;



    wifi.set_configuration(&Configuration::Client(wifi_credentials::get_wifi()))?;

        // Start Wifi
    wifi.start()?;

    // Connect Wifi
    wifi.connect()?;

    // Wait until the network interface is up
    wifi.wait_netif_up()?;

    // Print Out Wifi Connection Configuration
    while !wifi.is_connected().unwrap() {
        // Get and print connection configuration
        let config = wifi.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
    }

    Ok(wifi)

}



pub fn http_post(url: &str, headers: &[(&str, &str)], long_data: bool) -> anyhow::Result<Vec<u8>> {
    // HTTP Configuration
    // Create HTTPS Connection Handle
    let httpconnection = EspHttpConnection::new(&HttpConfig {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;

    // Create HTTPS Client
    let mut httpclient = Client::wrap(httpconnection);


    // Prepare request
    let request = httpclient.post(url, headers)?;

    // Log URL and type of request
    println!("-> POST {}", url);

    // Submit Request and Store Response
    let mut response: esp_idf_svc::http::client::Response<&mut EspHttpConnection> = request.submit()?;

    // HTTP Response Processing
    let status = response.status();
    println!("<- {}", status);

    // Read the received data chunk by chunk into an u8 buffer
    Ok(read_response_body(&mut response, long_data)?)

}

fn read_response_body(response: &mut esp_idf_svc::http::client::Response<&mut EspHttpConnection>, long_data: bool) -> anyhow::Result<Vec<u8>> {
    let mut buf: [u8; 1024] = [0; 1024];
    let mut total_bytes_read = 0;
    let mut body: Vec<u8> = Vec::new();
    if long_data {body.reserve(480*800/4);}
    

    loop {
        match response.read(&mut buf) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    break;
                }
                total_bytes_read += bytes_read;
                let data = &buf[..bytes_read];
                body.extend_from_slice(data);
            }
            Err(err) => {
                eprintln!("Error reading response: {}", err);
                break;
            }
        }
    }

    // Report the total number of bytes read
    println!("Total bytes read: {}", total_bytes_read);
    Ok(body)
}