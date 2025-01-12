#![no_std]
#![no_main]
#![macro_use]
#![feature(type_alias_impl_trait)]
#![deny(elided_lifetimes_in_paths)]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_stm32::pac;
use embassy_stm32::time::Hertz;
use embassy_time::Duration;

mod device;
mod iv;
mod lora_radio;
mod timer;

use defmt_rtt as _;
use device::*;
use lorawan::device::Device;
use lorawan::mac::region::channel_plan::dynamic::DynamicChannelPlan;
use lorawan::mac::region::eu868::EU868;
use lorawan::mac::types::Credentials;
use lorawan::mac::Mac;
#[cfg(debug_assertions)]
use panic_probe as _;
// release profile: minimize the binary size of the application
#[cfg(not(debug_assertions))]
use panic_reset as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(32_000_000),
            mode: HseMode::Bypass,
            prescaler: HsePrescaler::DIV1,
        });
        // config.rcc.mux = ClockSource::HSE;
        // config.rcc.pll = Some(Pll {
        //     source: PLLSource::HSE,
        //     prediv: PllPreDiv::DIV2,
        //     mul: PllMul::MUL6,
        //     divp: None,
        //     divq: Some(PllQDiv::DIV2), // PLL1_Q clock (32 / 2 * 6 / 2), used for RNG
        //     divr: Some(PllRDiv::DIV2), // sysclk 48Mhz clock (32 / 2 * 6 / 2)
        // });
    }
    let peripherals = embassy_stm32::init(config);

    pac::RCC.ccipr().modify(|w| w.set_rngsel(pac::rcc::vals::Rngsel::MSI));
    let mut device = LoraDevice::new(peripherals).await;
    let mut buffer = [0u8; 256];
    let mut mac = get_mac(&mut device);
    loop {
        while !mac.is_joined() {
            defmt::info!("JOINING");
            match mac.join(&mut device, &mut buffer).await {
                Ok(res) => defmt::info!("Network joined! {:?}", res),
                Err(e) => {
                    defmt::error!("Join failed {:?}", e);
                    embassy_time::Timer::after(Duration::from_secs(600)).await;
                }
            };
        }
        'sending: while mac.is_joined() {
            defmt::info!("SENDING");
            let send_res = mac.send(&mut device, &mut buffer, b"PING", 1, false, None).await;
            match send_res {
                Ok(Some((len, status))) => {
                    defmt::info!("Sent: Rx len: {} RSSI: {} SNR:{}", len, status.rssi, status.snr)
                }
                Ok(None) => defmt::info!("Sent: no downlink"),
                Err(e) => {
                    defmt::error!("{:?}", e);
                    if let lorawan::Error::Mac(lorawan::mac::Error::SessionExpired) = e {
                        defmt::info!("Session expired");
                        break 'sending;
                    };
                }
            }

            embassy_time::Timer::after(Duration::from_secs(300)).await;
        }
    }
}
pub fn get_mac(device: &mut LoraDevice<'static>) -> Mac<EU868, DynamicChannelPlan<EU868>> {
    pub const DEVICE_ID_PTR: *const u8 = 0x1FFF_7580 as _;
    let dev_eui: [u8; 8] = unsafe { *DEVICE_ID_PTR.cast::<[u8; 8]>() };
    let app_eui: [u8; 8] = [0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01];
    let app_key: [u8; 16] = [
        0x2B, 0x7E, 0x15, 0x16, 0x28, 0xAE, 0xD2, 0xA6, 0xAB, 0xF7, 0x15, 0x88, 0x09, 0xCF, 0x4F,
        0x3C,
    ];
    defmt::info!(
        "deveui:\t{:X}-{:X}-{:X}-{:X}-{:X}-{:X}-{:X}-{:X}",
        dev_eui[7],
        dev_eui[6],
        dev_eui[5],
        dev_eui[4],
        dev_eui[3],
        dev_eui[2],
        dev_eui[1],
        dev_eui[0]
    );

    let hydrate_res = device.hydrate_from_non_volatile(app_eui, dev_eui, app_key);
    match hydrate_res {
        Ok(_) => defmt::info!("credentials and configuration loaded from non volatile"),
        Err(_) => defmt::info!("credentials and configuration not found in non volatile"),
    };
    let (configuration, credentials) =
        hydrate_res.unwrap_or((Default::default(), Credentials::new(app_eui, dev_eui, app_key)));
    Mac::new(configuration, credentials)
}
