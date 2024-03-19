use std::mem::drop;
use anyhow::Result;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::nvs::EspNvs;
use esp_idf_sys::{esp_deep_sleep_start, esp_sleep_enable_timer_wakeup};


mod wifi_abstraction;
mod e_ink;
mod wifi_credentials;


fn main() -> Result<()> {
    if mainfunc().is_err() {
        println!("Error in mainfunc");
        println!("Going to sleep");
        unsafe{esp_sleep_enable_timer_wakeup(3600 * 1000000);}
        unsafe{esp_deep_sleep_start();}
    
    }
    Ok(())
}


fn mainfunc() -> Result<()> {
    esp_idf_sys::link_patches();

    // Configure Wifi
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let id_nvs = EspNvs::new(nvs.clone(), "id", true)?;



    //get id
    let mut id_option = id_nvs.get_u32("ID")?;
    let mut id = id_option.get_or_insert(0).clone();

    // Setup Wifi
    let modem: esp_idf_hal::modem::Modem = peripherals.modem;
    let mut wifi = wifi_abstraction::setup_wifi(modem, sysloop, nvs)?;

    // Configure E-Ink
    let pins: esp_idf_hal::gpio::Pins = peripherals.pins;
    let mut display: e_ink::EInk = e_ink::initialize_eink(pins)?;

    //check id 
    let res_id = wifi_abstraction::http_post(&(wifi_credentials::get_server()+ "api/id/check"), &[("DisplayID", &id.to_string())], false)?;
    if res_id[0] == '0' as u8 {
        let res_new_id = wifi_abstraction::http_post(&(wifi_credentials::get_server() + "api/id/new"), &[], false)?;
        id = String::from_utf8(res_new_id)?.parse::<u32>()?;
        id_nvs.set_u32("ID", id)?;
    }
    drop(res_id);

    let res_sleep_time = wifi_abstraction::http_post(&(wifi_credentials::get_server() + "api/sleep/requestTime"), &[("DisplayID", &id.to_string())], false)?;
    let sleep_time = String::from_utf8(res_sleep_time)?.parse::<u64>()?;
    unsafe{esp_sleep_enable_timer_wakeup(sleep_time * 1000000);}

    let res_update = wifi_abstraction::http_post(&(wifi_credentials::get_server() + "api/update/necessary"), &[("DisplayID", &id.to_string())], false)?;
    if res_update[0] == '0' as u8 {
        unsafe{esp_deep_sleep_start();}
    }
    drop(res_update);

    
    let res = wifi_abstraction::http_post(&(wifi_credentials::get_server() + "api/btmp/full"), &[("DisplayID", &id.to_string())], true)?;
    //delete wifi
    wifi.disconnect()?;
    wifi.stop()?;

    


    println!("uploading data");
    (display.calls.upload_char_vector)(&mut display.spi, res)?;
    println!("upload done");
    (display.calls.upload_rw)(&mut display.spi, &[0x00; 48000])?; 

    println!("refreshing");
    (display.calls.refresh)(&mut display.spi)?;
    println!("refresh done");
    println!("going to sleep");
    unsafe{esp_deep_sleep_start()};
    #[allow(unreachable_code)]
    Ok(())
}

