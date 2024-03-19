use anyhow::{Ok, Result};
use esp_idf_hal::gpio::{self, PinDriver, Pins};

pub struct EInkSpi {
    sck: PinDriver<'static, gpio::Gpio13, gpio::Output>,
    din: PinDriver<'static, gpio::Gpio14, gpio::Output>,
    cs: PinDriver<'static, gpio::Gpio15, gpio::Output>,
    busy: PinDriver<'static, gpio::Gpio25, gpio::Input>,
    rst: PinDriver<'static, gpio::Gpio26, gpio::Output>,
    dc: PinDriver<'static, gpio::Gpio27, gpio::Output>,
}

pub struct EInkCalls{
    pub upload_bw: fn(&mut EInkSpi, &[u8]) -> Result<()>,
    pub upload_rw: fn(&mut EInkSpi, &[u8]) -> Result<()>,
    pub reset: fn(&mut EInkSpi) -> Result<()>,
    pub refresh: fn(&mut EInkSpi) -> Result<()>,
    pub upload_char_vector: fn(&mut EInkSpi, Vec<u8>) -> Result<()>,
}
pub struct EInk {
    pub spi: EInkSpi,
    pub calls: EInkCalls,
}


fn initialize_spi(pins: Pins) -> Result<EInkSpi> {
    let mut sck: PinDriver<'static, gpio::Gpio13, gpio::Output> = PinDriver::output(pins.gpio13)?;
    let din: PinDriver<'static, gpio::Gpio14, gpio::Output> = PinDriver::output(pins.gpio14)?;
    let mut cs: PinDriver<'static, gpio::Gpio15, gpio::Output> = PinDriver::output(pins.gpio15)?;
    let busy: PinDriver<'static, gpio::Gpio25, gpio::Input> = PinDriver::input(pins.gpio25)?;
    let rst: PinDriver<'static, gpio::Gpio26, gpio::Output> = PinDriver::output(pins.gpio26)?;
    let dc: PinDriver<'static, gpio::Gpio27, gpio::Output> = PinDriver::output(pins.gpio27)?;

    cs.set_high()?;
    sck.set_low()?;
    println!("SPI initialized");


    Ok(EInkSpi {
        sck,
        din,
        cs,
        busy,
        rst,
        dc,
    })

}

fn transfer_byte(spi: &mut EInkSpi, data: u8) -> Result<()> {
    spi.cs.set_low()?;
    for i in 0..8 {
        if data & (1 << (7 - i)) != 0 {
            spi.din.set_high()?;
        } else {
            spi.din.set_low()?;
        }
        spi.sck.set_high()?;
        spi.sck.set_low()?;
    }
    spi.cs.set_high()?;
    Ok(())
}

fn send_command(spi: &mut EInkSpi, byte: u8) -> Result<()> {
    spi.dc.set_low()?;
    transfer_byte(spi, byte)
}

fn send_data(spi: &mut EInkSpi, data: &[u8]) -> Result<()> {
    spi.dc.set_high()?;
    for byte in data {
        transfer_byte(spi, *byte)?;
    }
    Ok(())
}

fn send_command_data(spi: &mut EInkSpi, command: u8, data: &[u8]) -> Result<()> {
    send_command(spi, command)?;
    send_data(spi, data)
}


fn check_busy(spi: &mut EInkSpi) -> Result<bool> {
    let busy: bool = spi.busy.is_high();
    Ok(busy)
}

fn reset(spi: &mut EInkSpi) -> Result<()> {
    spi.rst.set_high()?;
    std::thread::sleep(std::time::Duration::from_millis(200));
    spi.rst.set_low()?;
    std::thread::sleep(std::time::Duration::from_millis(5));
    spi.rst.set_high()?;
    std::thread::sleep(std::time::Duration::from_millis(200));
    Ok(())
}

pub fn refresh(spi: &mut EInkSpi) -> Result<()> {
    send_command(spi, 0x12)?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    while !check_busy(spi)? {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}

pub fn upload_bw(spi: &mut EInkSpi, data: &[u8]) -> Result<()> {
    send_command_data(spi, 0x10, data)?;
    send_command(spi, 0x11)?;
    Ok(())
}

pub fn upload_char_vector(spi: &mut EInkSpi, mut data: Vec<u8>) -> Result<()> {
    if data.len() % 2 != 0{
        data.push(0x00);
    }
    send_command(spi, 0x10)?;
    spi.dc.set_high()?;
    for i in 0..data.len() / 2 {

        let byte = ((data[i*2] as u8) << 4 & 0xf0)| ((data[i*2 + 1] as u8) & 0x0f);
        transfer_byte(spi, byte)?;
    }
    send_command(spi, 0x11)?;

    Ok(())
}

pub fn upload_rw(spi: &mut EInkSpi, data: &[u8]) -> Result<()> {
    send_command_data(spi, 0x13, data)?;
    send_command(spi, 0x11)?;
    Ok(())
}

pub fn initialize_eink(pins: Pins) -> Result<EInk> {
    let mut spi = initialize_spi(pins)?;
    


    reset(&mut spi)?;
    println!("Reset done");
    //wait until busy is low
    while !check_busy(&mut spi)? {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("starting power");
    send_command(&mut spi, 0x04)?; // Power on

    while !check_busy(&mut spi)? {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("power on done");

    let calls = EInkCalls {
        upload_bw,
        upload_rw,
        reset,
        refresh,
        upload_char_vector,
    };

    Ok(EInk {
        spi,
        calls,
    })
}