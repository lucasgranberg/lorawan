//! API for the LoRaWAN MAC layer using device properties specified by the caller.

use core::fmt::Debug;

pub mod region;
pub mod types;
use core::{
    cmp::{max, min},
    marker::PhantomData,
};

use self::region::{
    channel_plan::{Channel, ChannelPlan, NUM_OF_CHANNEL_BLOCKS},
    Region,
};

use crate::device::DeviceSpecs;
use crate::{
    device::types::{RfConfig, TxConfig},
    device::{radio_buffer::RadioBuffer, rng::Rng, timer::Timer, Device},
};
use encoding::parser::AsPhyPayloadBytes;
use encoding::{
    creator::{DataPayloadCreator, JoinRequestCreator},
    default_crypto::DefaultFactory,
    maccommandcreator::{
        DevStatusAnsCreator, DlChannelAnsCreator, DutyCycleAnsCreator, LinkADRAnsCreator,
        NewChannelAnsCreator, RXParamSetupAnsCreator, RXTimingSetupAnsCreator,
        TXParamSetupAnsCreator, UplinkMacCommandCreator,
    },
    maccommands::{DLSettings, DownlinkMacCommand, MacCommandIterator, SerializableMacCommand},
    parser::{parse_with_factory, DataHeader, DevNonce, FCtrl, FRMPayload, PhyPayload},
};
#[cfg(not(feature = "defmt"))]
macro_rules! trace {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            let _ = ($( & $x ),*);
        }
    };
}
#[cfg(feature = "defmt")]
use defmt::trace;
#[cfg(not(feature = "defmt"))]
macro_rules! debug {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            let _ = ($( & $x ),*);
        }
    };
}
#[cfg(feature = "defmt")]
use defmt::debug;

use futures::pin_mut;
use heapless::Vec;
use lora_modulation::{BaseBandModulationParams, CodingRate};
use lora_phy::mod_params::{PacketParams, PacketStatus};
use types::*;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
pub enum Error {
    UnsupportedDataRate,
    InvalidMic,
    InvalidDevAddr,
    InvalidPayloadType,
    NoResponse,
    NetworkNotJoined,
    SessionExpired,
    FOptsFull,
    NoValidChannelFound,
}
impl<D> From<Error> for super::Error<D>
where
    D: Device,
{
    fn from(value: Error) -> Self {
        Self::Mac(value)
    }
}

/// Composition of properties needed to guide LoRaWAN MAC layer processing, supporting the LoRaWAN MAC API.
#[repr(C)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Mac<R, C>
where
    R: Region,
    C: ChannelPlan<R> + Default,
{
    pub(crate) session: Option<Session>,
    pub(crate) channel_plan: C,
    pub(crate) region: PhantomData<R>,
    pub(crate) uplink_cmds: Vec<UplinkMacCommandCreator, 15>,
    pub(crate) ack_next: bool,
    pub(crate) configuration: Configuration,
    pub(crate) credentials: Credentials,
    pub(crate) adr_ack_cnt: u8,
}

impl<R, C> Mac<R, C>
where
    R: region::Region,
    C: ChannelPlan<R> + Default,
{
    /// Creation.
    pub fn new(configuration: Configuration, credentials: Credentials) -> Self {
        Self {
            session: None,
            channel_plan: Default::default(),
            region: PhantomData,
            uplink_cmds: Vec::new(),
            ack_next: false,
            configuration,
            credentials,
            adr_ack_cnt: 0,
        }
    }

    /// Get the minimum frequency, perhaps unique to the given end device.
    fn min_frequency<D: DeviceSpecs>() -> u32 {
        match D::min_frequency() {
            Some(device_min_frequency) => max(device_min_frequency, R::min_frequency()),
            None => R::min_frequency(),
        }
    }
    /// Get the maximum frequency, perhaps unique to the given end device
    fn max_frequency<D: DeviceSpecs>() -> u32 {
        match D::max_frequency() {
            Some(device_max_frequency) => min(device_max_frequency, R::max_frequency()),
            None => R::max_frequency(),
        }
    }
    /// Is the frequency within range for the given end device.
    fn validate_frequency<D: DeviceSpecs>(frequency: u32) -> bool {
        let frequency_range = Self::min_frequency::<D>()..=Self::max_frequency::<D>();
        frequency_range.contains(&frequency)
    }

    /// Get the maximum uplink data rate, perhaps unique to the given end device.
    fn max_data_rate<D: DeviceSpecs>() -> DR {
        match D::max_data_rate() {
            Some(device_max_data_rate) => {
                min(device_max_data_rate as u8, R::ul_data_rate_range().1 as u8).try_into().unwrap()
            }
            None => R::ul_data_rate_range().1,
        }
    }

    /// Get the minumum uplink data rate, perhaps unique to the given end device.
    fn min_data_rate<D: DeviceSpecs>() -> DR {
        match D::min_data_rate() {
            Some(device_min_data_rate) => {
                max(device_min_data_rate as u8, R::ul_data_rate_range().0 as u8).try_into().unwrap()
            }
            None => R::ul_data_rate_range().0,
        }
    }

    /// Is the RX1 data rate offset within range for the given end device?
    fn validate_rx1_data_rate_offset<D: DeviceSpecs>(rx1_dr_offset: u8) -> bool {
        R::get_rx1_dr(Self::min_data_rate::<D>(), rx1_dr_offset).is_ok()
            && R::get_rx1_dr(Self::max_data_rate::<D>(), rx1_dr_offset).is_ok()
    }

    /// Is the uplink data rate within range for the given end device?
    fn validate_data_rate<D: DeviceSpecs>(dr: u8) -> bool {
        DR::try_from(dr).unwrap().in_range((Self::min_data_rate::<D>(), Self::max_data_rate::<D>()))
    }

    /// Are the downlink data rate settings in range for the given end device?
    fn validate_dl_settings<D: DeviceSpecs>(dl_settings: DLSettings) -> (bool, bool) {
        let rx1_data_rate_offset_ack =
            Self::validate_rx1_data_rate_offset::<D>(dl_settings.rx1_dr_offset());
        let rx2_data_rate_ack = Self::validate_data_rate::<D>(dl_settings.rx2_data_rate());
        (rx1_data_rate_offset_ack, rx2_data_rate_ack)
    }

    /// Get the maximum EIRP for the end device.
    fn max_eirp<D: DeviceSpecs>() -> i8 {
        if let Some(device_max_eirp) = D::max_eirp() {
            min(R::max_eirp(), device_max_eirp)
        } else {
            R::max_eirp()
        }
    }

    /// Get the transmission power based on the frame type.
    fn get_tx_pwr<D: DeviceSpecs>(frame: Frame, configuration: &Configuration) -> i8 {
        match frame {
            Frame::Join => Self::max_eirp::<D>(),
            Frame::Data => configuration.tx_power.unwrap_or(Self::max_eirp::<D>()),
        }
    }

    /// Has a session been established bteween the end device and a network server?
    pub fn is_joined(&self) -> bool {
        if let Some(session) = &self.session {
            !session.is_expired()
        } else {
            false
        }
    }

    fn handle_dl_settings(&mut self, dl_settings: DLSettings) -> Result<(), crate::mac::Error> {
        self.configuration.rx1_data_rate_offset = Some(dl_settings.rx1_dr_offset());
        let rx2_data_rate: DR = dl_settings
            .rx2_data_rate()
            .try_into()
            .map_err(|_| crate::mac::Error::UnsupportedDataRate)?;
        self.configuration.rx2_data_rate = Some(rx2_data_rate);
        Ok(())
    }

    fn tx_data_rate(&self) -> DR {
        self.configuration.tx_data_rate.unwrap_or(R::default_data_rate())
    }

    fn rx1_data_rate_offset(&self) -> u8 {
        self.configuration.rx1_data_rate_offset.unwrap_or(R::default_rx1_data_rate_offset())
    }

    fn rx1_data_rate(&self, tx_dr: DR) -> DR {
        let offset = self.rx1_data_rate_offset();
        R::get_rx1_dr(tx_dr, offset).unwrap_or(R::default_data_rate())
    }

    fn rx2_data_rate(&self, frame: &Frame) -> DR {
        match frame {
            Frame::Join => R::default_rx2_data_rate(),
            Frame::Data => self.configuration.rx2_data_rate.unwrap_or(R::default_rx2_data_rate()),
        }
    }

    fn adr_ack_cnt_increment(&mut self) {
        if let Some(val) = self.adr_ack_cnt.checked_add(1) {
            self.adr_ack_cnt = val
        };
    }

    fn adr_back_off(&mut self) {
        if self.adr_ack_cnt >= (R::default_adr_ack_limit() + R::default_adr_ack_delay()) {
            // Start back off sequence
            if self.configuration.tx_power.is_some() {
                // First reset tx_power to default
                self.configuration.tx_power = None;
            } else {
                // Next decrese tx_power until it reaches default
                if self.configuration.tx_data_rate.is_some() {
                    self.configuration.tx_data_rate =
                        R::next_adr_data_rate(self.configuration.tx_data_rate);
                } else {
                    self.configuration.number_of_transmissions = 1;
                    self.channel_plan.reactivate_channels();
                }
            }
        }
    }

    pub(crate) fn create_join_request<const N: usize>(&self, buf: &mut RadioBuffer<N>) {
        buf.clear();

        let mut phy: JoinRequestCreator<[u8; 23], DefaultFactory> = JoinRequestCreator::default();

        let devnonce = self.credentials.dev_nonce;

        phy.set_app_eui(self.credentials.app_eui)
            .set_dev_eui(self.credentials.dev_eui)
            .set_dev_nonce(&devnonce.to_le_bytes());
        let vec = phy.build(&self.credentials.app_key);

        buf.extend_from_slice(vec).unwrap();
    }

    fn get_rx_windows(&self, frame: Frame) -> RxWindows {
        const ADJUST: u16 = 15;
        match frame {
            Frame::Join => RxWindows {
                rx1_open: R::default_join_accept_delay1() - ADJUST, // current observed duration to prepare to receive ranges from 0 to 13 ms ???
                rx2_open: R::default_join_accept_delay2() - ADJUST,
            },
            Frame::Data => {
                let rx1_delay: u16 = self
                    .configuration
                    .rx_delay
                    .map(|delay| delay as u16 * 1000)
                    .unwrap_or(R::default_rx_delay());
                let rx2_delay = rx1_delay + 1000;
                RxWindows {
                    rx1_open: rx1_delay - ADJUST, // current observed duration to prepare to receive ranges from 0 to 13 ms ???
                    rx2_open: rx2_delay - ADJUST,
                }
            }
        }
    }

    fn create_tx_config<D: Device>(
        &self,
        frame: Frame,
        channel: &C::Channel,
        dr: DR,
    ) -> Result<TxConfig, crate::Error<D>> {
        let pw = Self::get_tx_pwr::<D>(frame, &self.configuration);
        let data_rate = R::convert_data_rate(dr)?;
        let tx_config = TxConfig {
            pw,
            rf: RfConfig {
                frequency: channel.get_ul_frequency(),
                coding_rate: CodingRate::_4_5,
                data_rate,
            },
        };
        Ok(tx_config)
    }

    fn create_rf_config<D: Device>(
        &self,
        frame: &Frame,
        window: &Window,
        data_rate: DR,
        channel: &C::Channel,
    ) -> Result<RfConfig, crate::Error<D>> {
        let data_rate = match window {
            Window::_1 => self.rx1_data_rate(data_rate),
            Window::_2 => self.rx2_data_rate(frame),
        };
        let data_rate = R::convert_data_rate(data_rate)?;
        let rf_config = match (frame, window) {
            (Frame::Join, Window::_1) => RfConfig {
                frequency: channel.get_dl_frequency(),
                coding_rate: CodingRate::_4_5,
                data_rate,
            },
            (Frame::Join, Window::_2) => RfConfig {
                frequency: R::default_rx2_frequency(),
                coding_rate: CodingRate::_4_5,
                data_rate,
            },
            (Frame::Data, Window::_1) => RfConfig {
                frequency: channel.get_dl_frequency(),
                coding_rate: CodingRate::_4_5,
                data_rate,
            },
            (Frame::Data, Window::_2) => RfConfig {
                frequency: R::default_rx2_frequency(),
                coding_rate: CodingRate::_4_5,
                data_rate,
            },
        };
        Ok(rf_config)
    }

    async fn prepare_for_rx<D: Device>(
        &self,
        rf_config: &RfConfig,
        device: &mut D,
    ) -> Result<PacketParams, crate::Error<D>> {
        let mdltn_params = device
            .radio()
            .create_modulation_params(
                rf_config.data_rate.spreading_factor,
                rf_config.data_rate.bandwidth,
                rf_config.coding_rate,
                rf_config.frequency,
            )
            .map_err(crate::device::Error::Radio)?;

        let rx_pkt_params = device
            .radio()
            .create_rx_packet_params(8, false, 255, true, true, &mdltn_params)
            .map_err(crate::device::Error::Radio)?;
        let bb = BaseBandModulationParams::new(
            rf_config.data_rate.spreading_factor,
            rf_config.data_rate.bandwidth,
            rf_config.coding_rate,
        );
        const PREAMBLE_SYMBOLS: u16 = 13; // 12.25
        let num_symbols = PREAMBLE_SYMBOLS + bb.delay_in_symbols(100);
        let rx_mode = lora_phy::RxMode::Single(num_symbols);

        device
            .radio()
            .prepare_for_rx(rx_mode, &mdltn_params, &rx_pkt_params)
            .await
            .map_err(crate::device::Error::Radio)?;
        Ok(rx_pkt_params)
    }

    fn handle_downlink_macs<D: Device>(
        &mut self,
        device: &mut D,
        packet_status: PacketStatus,
        cmds: MacCommandIterator<'_, DownlinkMacCommand<'_>>,
    ) -> Result<(), crate::Error<D>> {
        let mut channel_mask = self.channel_plan.get_channel_mask();
        let mut cmd_iter = cmds.into_iter().peekable();
        while let Some(cmd) = cmd_iter.next() {
            trace!("hadling command {:?}", cmd);
            let res: Option<UplinkMacCommandCreator> = match cmd {
                DownlinkMacCommand::LinkCheckAns(payload) => {
                    device.handle_link_check(payload.gateway_count(), payload.margin());
                    None
                }
                DownlinkMacCommand::DeviceTimeAns(payload) => {
                    device.handle_device_time(payload.seconds(), payload.nano_seconds());
                    None
                }
                DownlinkMacCommand::LinkADRReq(payload) => {
                    let mut ans = LinkADRAnsCreator::new();
                    let tx_power_res = R::modify_dbm(
                        payload.tx_power(),
                        self.configuration.tx_power,
                        R::max_eirp(),
                    );
                    let data_rate_res: Result<Option<DR>, ()> = if payload.data_rate() == 0xF {
                        Ok(self.configuration.tx_data_rate)
                    } else {
                        DR::try_from(payload.data_rate()).map(Some)
                    };

                    let channel_mask_res = self.channel_plan.handle_channel_mask(
                        &mut channel_mask,
                        payload.channel_mask(),
                        payload.redundancy().channel_mask_control(),
                    );

                    ans.set_tx_power_ack(tx_power_res.is_ok());
                    ans.set_data_rate_ack(data_rate_res.is_ok());
                    ans.set_channel_mask_ack(channel_mask_res.is_ok());
                    // check if next command is also a LinkADRReq, if not process the atomic block
                    match cmd_iter.peek() {
                        Some(DownlinkMacCommand::LinkADRReq(_)) => (),
                        _ => {
                            // The end-device SHALL respond to all LinkADRReq commands
                            // with a LinkADRAns indicating which command elements were accepted and which were
                            // rejected. This behavior differs from when the uplink ADR bit is set, in which case the end-
                            // device accepts or rejects the entire command.
                            if !device.adaptive_data_rate_enabled()
                                || (tx_power_res.is_ok()
                                    && data_rate_res.is_ok()
                                    && channel_mask_res.is_ok())
                            {
                                if let Ok(new_tx_power) = tx_power_res {
                                    self.configuration.tx_power = new_tx_power
                                }
                                if let Ok(new_data_rate) = data_rate_res {
                                    self.configuration.tx_data_rate = new_data_rate
                                }
                                if channel_mask_res.is_ok() {
                                    self.configuration.number_of_transmissions =
                                        payload.redundancy().number_of_transmissions();
                                    if self.configuration.number_of_transmissions == 0 {
                                        self.configuration.number_of_transmissions = 1;
                                    }
                                }
                            }
                            //reset channel mask to match actual status
                            channel_mask = self.channel_plan.get_channel_mask();
                        }
                    }

                    Some(UplinkMacCommandCreator::LinkADRAns(ans))
                }
                DownlinkMacCommand::DutyCycleReq(payload) => {
                    self.configuration.max_duty_cycle = payload.max_duty_cycle();
                    Some(UplinkMacCommandCreator::DutyCycleAns(DutyCycleAnsCreator::new()))
                }
                DownlinkMacCommand::RXParamSetupReq(payload) => {
                    let mut ans = RXParamSetupAnsCreator::new();
                    let (mut rx1_data_rate_offset_ack, mut rx2_data_rate_ack) =
                        Self::validate_dl_settings::<D>(payload.dl_settings());
                    let channel_ack =
                        self.channel_plan.validate_frequency(payload.frequency().value()).is_ok();
                    if channel_ack && rx1_data_rate_offset_ack && rx2_data_rate_ack {
                        if self.handle_dl_settings(payload.dl_settings()).is_err() {
                            rx1_data_rate_offset_ack = false;
                            rx2_data_rate_ack = false;
                        } else {
                            self.configuration.rx2_frequency = Some(payload.frequency().value());
                        }
                    }
                    ans.set_rx1_data_rate_offset_ack(rx1_data_rate_offset_ack);
                    ans.set_rx2_data_rate_ack(rx2_data_rate_ack);
                    Some(UplinkMacCommandCreator::RXParamSetupAns(ans))
                }
                DownlinkMacCommand::DevStatusReq(_) => {
                    let mut ans = DevStatusAnsCreator::new();
                    match device.battery_level() {
                        Some(battery_level) => ans.set_battery((battery_level * 253.0) as u8 + 1),
                        None => ans.set_battery(255),
                    };
                    ans.set_margin(packet_status.snr as i8)
                        .map_err(|_| crate::Error::<D>::Encoding)?;
                    Some(UplinkMacCommandCreator::DevStatusAns(ans))
                }
                DownlinkMacCommand::NewChannelReq(payload) => {
                    if (payload.channel_index() as usize) < R::default_channels(true) {
                        None //silently ignore if default channel
                    } else {
                        let data_rate_range_ack = Self::validate_data_rate::<D>(
                            payload.data_rate_range().min_data_rate(),
                        ) && Self::validate_data_rate::<D>(
                            payload.data_rate_range().max_data_rate(),
                        ) && payload.data_rate_range().min_data_rate()
                            < payload.data_rate_range().max_data_rate();

                        let channel_frequency_ack = payload.frequency().value() == 0
                            || Self::validate_frequency::<D>(payload.frequency().value());

                        let mut ans = NewChannelAnsCreator::new();
                        ans.set_channel_frequency_ack(channel_frequency_ack);
                        ans.set_data_rate_range_ack(data_rate_range_ack);
                        if data_rate_range_ack && channel_frequency_ack {
                            match self.channel_plan.handle_new_channel_req(payload) {
                                Ok(_) => ans.set_channel_frequency_ack(true),
                                Err(_) => ans.set_channel_frequency_ack(false),
                            };
                        }
                        Some(UplinkMacCommandCreator::NewChannelAns(ans))
                    }
                }
                DownlinkMacCommand::DlChannelReq(payload) => {
                    let mut ans = DlChannelAnsCreator::new();
                    let mut channel_frequency_ack =
                        Self::validate_frequency::<D>(payload.frequency().value());
                    //let mut uplink_frequency_exists_ack = false;
                    let uplink_frequency_exists_ack = self
                        .channel_plan
                        .check_uplink_frequency_exists(payload.channel_index() as usize);
                    if channel_frequency_ack {
                        channel_frequency_ack =
                            self.channel_plan.handle_dl_channel_req(payload).is_ok()
                    }
                    ans.set_uplink_frequency_exists_ack(uplink_frequency_exists_ack);
                    ans.set_channel_frequency_ack(channel_frequency_ack);
                    Some(UplinkMacCommandCreator::DlChannelAns(ans))
                }
                DownlinkMacCommand::RXTimingSetupReq(payload) => {
                    let delay = match payload.delay() {
                        0 => 1,
                        _ => payload.delay(),
                    };
                    self.configuration.rx_delay = Some(delay);
                    Some(UplinkMacCommandCreator::RXTimingSetupAns(RXTimingSetupAnsCreator::new()))
                }
                DownlinkMacCommand::TXParamSetupReq(_) => {
                    if R::supports_tx_param_setup() {
                        let ans = TXParamSetupAnsCreator::new();
                        let _ret = Some(UplinkMacCommandCreator::TXParamSetupAns(ans));
                        //persist_configuration = true;
                        todo!("TXParamSetupReq not implemented yet");
                    } else {
                        None
                    }
                }
            };
            if let Some(uplink_cmd) = res {
                trace!("answer {:?}", uplink_cmd);
                self.uplink_cmds.push(uplink_cmd).map_err(|_| crate::mac::Error::FOptsFull)?
            }
        }
        Ok(())
    }

    async fn rx_with_timeout<'a, D: Device>(
        &self,
        frame: Frame,
        device: &mut D,
        radio_buffer: &'a mut RadioBuffer<256>,
        data_rate: DR,
        channel: &C::Channel,
    ) -> Result<Option<(u8, PacketStatus)>, crate::Error<D>> {
        let windows = self.get_rx_windows(frame);

        let open_rx1_fut = device
            .timer()
            .at(windows.get_open(&Window::_1) as u64)
            .map_err(crate::device::Error::Timer)?;
        let open_rx2_fut = device
            .timer()
            .at(windows.get_open(&Window::_2) as u64)
            .map_err(crate::device::Error::Timer)?;
        pin_mut!(open_rx1_fut);
        pin_mut!(open_rx2_fut);

        radio_buffer.clear();

        let rf_config = self.create_rf_config(&frame, &Window::_1, data_rate, channel)?;
        debug!("rf config RX1 {:?}", rf_config);
        open_rx1_fut.await;
        let packet_params = self.prepare_for_rx(&rf_config, device).await?;

        match device.radio().rx(&packet_params, radio_buffer.as_raw_slice()).await {
            Ok(ret) => {
                return Ok(Some(ret));
            }
            // Bail on error other than timeout ???
            Err(_e) => {}
        }

        let rf_config = self.create_rf_config(&frame, &Window::_2, data_rate, channel)?;
        debug!("rf config RX2 {:?}", rf_config);
        let packet_params = self.prepare_for_rx(&rf_config, device).await?;
        open_rx2_fut.await;

        match device.radio().rx(&packet_params, radio_buffer.as_raw_slice()).await {
            Ok(ret) => Ok(Some(ret)),
            Err(e) => Err(crate::Error::Device(crate::device::Error::Radio(e))),
        }
    }

    fn prepare_buffer<D: Device>(
        &mut self,
        data: &[u8],
        fport: u8,
        confirmed: bool,
        radio_buffer: &mut RadioBuffer<256>,
        device: &D,
    ) -> Result<u32, crate::Error<D>> {
        if let Some(session) = &self.session {
            // check if FCnt is used up
            if session.fcnt_up() == (0xFFFF + 1) {
                // signal that the session is expired
                return Err(crate::Error::Mac(crate::mac::Error::SessionExpired));
            }
            let fcnt = session.fcnt_up();
            let mut phy = DataPayloadCreator::new();

            let mut fctrl = FCtrl(0x0, true);
            if device.adaptive_data_rate_enabled() {
                fctrl.set_adr();
            }

            if self.ack_next {
                fctrl.set_ack();
            }

            if self.adr_ack_cnt >= R::default_adr_ack_limit() {
                fctrl.set_adr_ack_req();
            }

            phy.set_confirmed(confirmed)
                .set_uplink(true)
                .set_fctrl(&fctrl)
                .set_f_port(fport)
                .set_dev_addr(*session.devaddr())
                .set_fcnt(fcnt);

            let mut dyn_cmds: Vec<&dyn SerializableMacCommand, 8> = Vec::new();
            for cmd in self.uplink_cmds.iter() {
                if let Err(_e) = dyn_cmds.push(cmd) {
                    panic!("dyn_cmds too small compared to cmds")
                }
            }
            let packet = phy
                .build(data, &dyn_cmds, session.newskey(), session.appskey())
                .map_err(|_| crate::Error::<D>::Encoding)?;
            trace!("TX: {=[u8]:#02X}", packet);
            radio_buffer.clear();
            radio_buffer.extend_from_slice(packet).map_err(crate::device::Error::RadioBuffer)?;
            Ok(fcnt)
        } else {
            Err(crate::Error::Mac(crate::mac::Error::NetworkNotJoined))
        }
    }

    async fn send_buffer<D: Device>(
        &self,
        device: &mut D,
        radio_buffer: &mut RadioBuffer<256>,
        frame: Frame,
    ) -> Result<Option<(u8, PacketStatus)>, crate::Error<D>> {
        let tx_buffer = radio_buffer.clone();

        for trans_index in 0..self.configuration.number_of_transmissions {
            let channels = self.get_send_channels(device, frame)?;
            for channel in channels {
                if let Some(chn) = channel {
                    let tx_data_rate = R::override_ul_data_rate_if_necessary(
                        self.tx_data_rate(),
                        frame,
                        chn.get_ul_frequency(),
                    );
                    let tx_config = self.create_tx_config(frame, &chn, tx_data_rate)?;
                    let mdltn_params = device
                        .radio()
                        .create_modulation_params(
                            tx_config.rf.data_rate.spreading_factor,
                            tx_config.rf.data_rate.bandwidth,
                            tx_config.rf.coding_rate,
                            tx_config.rf.frequency,
                        )
                        .map_err(crate::device::Error::Radio)?;
                    let mut tx_pkt_params = device
                        .radio()
                        .create_tx_packet_params(8, false, true, false, &mdltn_params)
                        .map_err(crate::device::Error::Radio)?;

                    trace!("tx config {:?}", tx_config);
                    device
                        .radio()
                        .prepare_for_tx(
                            &mdltn_params,
                            &mut tx_pkt_params,
                            tx_config.pw as i32,
                            tx_buffer.as_ref(),
                        )
                        .await
                        .map_err(crate::device::Error::Radio)?;
                    device.radio().tx().await.map_err(crate::device::Error::Radio)?;
                    device.timer().reset();

                    match self
                        .rx_with_timeout(frame, device, radio_buffer, tx_data_rate, &chn)
                        .await
                    {
                        Ok(Some((num_read, rx_quality))) => {
                            radio_buffer.inc(num_read as usize);
                            return Ok(Some((num_read, rx_quality)));
                        }
                        Ok(None) => {
                            if frame == Frame::Data {
                                if (trans_index + 1) >= self.configuration.number_of_transmissions {
                                    return Ok(None);
                                } else {
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            if frame == Frame::Data {
                                if (trans_index + 1) >= self.configuration.number_of_transmissions {
                                    return Err(e);
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }

                // Delay for a random amount of time between 1 and 2 seconds ???
                let random = device.rng().next_u32().map_err(crate::device::Error::Rng)?;
                let delay_ms = 1000 + (random % 1000);
                device.timer().reset();
                let delay_fut =
                    device.timer().at(delay_ms as u64).map_err(crate::device::Error::Timer)?;
                delay_fut.await;
            }
        }
        Ok(None)
    }

    /// Establish a session between the end device and a network server.
    pub async fn join<'a, D: Device>(
        &'a mut self,
        device: &'a mut D,
        radio_buffer: &'a mut RadioBuffer<256>,
    ) -> Result<(), crate::Error<D>> {
        self.credentials.incr_dev_nonce();
        device
            .persist_to_non_volatile(&self.configuration, &self.credentials)
            .map_err(crate::device::Error::NonVolatileStore)?;
        self.create_join_request(radio_buffer);
        let rx_res = self.send_buffer(device, radio_buffer, Frame::Join).await?;
        if rx_res.is_some() {
            match parse_with_factory(radio_buffer.as_mut(), DefaultFactory)
                .map_err(|_| crate::Error::<D>::Encoding)?
            {
                PhyPayload::JoinAccept(encoding::parser::JoinAcceptPayload::Encrypted(
                    encrypted,
                )) => {
                    let decrypted = encrypted.decrypt(&self.credentials.app_key);
                    if decrypted.validate_mic(&self.credentials.app_key) {
                        let session = Session::derive_new(
                            &decrypted,
                            DevNonce::<[u8; 2]>::new(self.credentials.dev_nonce.to_le_bytes())
                                .unwrap(),
                            &self.credentials,
                        );
                        trace!("msg {=[u8]:02X}", decrypted.as_bytes());
                        trace!("new {=[u8]:02X}", session.newskey().inner().0);
                        trace!("app {=[u8]:02X}", session.appskey().inner().0);
                        trace!("rx1 {:?}", decrypted.dl_settings().rx1_dr_offset());
                        trace!("rx2 {:?}", decrypted.dl_settings().rx2_data_rate());
                        // trace!("rx2 {:?}", decrypted.c_f_list());
                        self.session.replace(session);
                        self.adr_ack_cnt = 0;

                        let (rx1_data_rate_offset_ack, rx2_data_rate_ack) =
                            Self::validate_dl_settings::<D>(decrypted.dl_settings());
                        trace!("{}{}", rx1_data_rate_offset_ack, rx2_data_rate_ack);
                        if rx1_data_rate_offset_ack && rx2_data_rate_ack {
                            self.handle_dl_settings(decrypted.dl_settings())?
                        }

                        let delay = match decrypted.rx_delay() {
                            0 => 1,
                            _ => decrypted.rx_delay(),
                        };
                        self.configuration.rx_delay = Some(delay);
                        if let Some(cf_list) = decrypted.c_f_list() {
                            self.channel_plan.handle_cf_list(cf_list)?;
                        }
                        device
                            .persist_to_non_volatile(&self.configuration, &self.credentials)
                            .map_err(crate::device::Error::NonVolatileStore)?;
                        Ok(())
                    } else {
                        Err(crate::Error::Mac(crate::mac::Error::InvalidMic))
                    }
                }
                _ => Err(crate::Error::Mac(crate::mac::Error::InvalidPayloadType)),
            }
        } else {
            Err(crate::Error::Mac(crate::mac::Error::NoResponse))
        }
    }

    /// Send data from the end device to a network server on an established session.
    pub async fn send<D: Device>(
        &mut self,
        device: &mut D,
        radio_buffer: &mut RadioBuffer<256>,
        data: &[u8],
        fport: u8,
        mut confirmed: bool,
        rx: Option<&mut [u8]>,
    ) -> Result<Option<(u8, PacketStatus)>, crate::Error<D>> {
        if let Some(ref mut session_data) = self.session {
            if !session_data.is_expired() {
                session_data.fcnt_up_increment();
                if device.adaptive_data_rate_enabled() {
                    self.adr_ack_cnt_increment();
                    self.adr_back_off();
                }
            } else {
                return Err(crate::Error::Mac(crate::mac::Error::SessionExpired));
            }
        } else {
            return Err(crate::Error::Mac(crate::mac::Error::NetworkNotJoined));
        }
        if !self.uplink_cmds.is_empty() {
            confirmed = true;
        }
        self.prepare_buffer(data, fport, confirmed, radio_buffer, device)?;
        let rx_res = self.send_buffer(device, radio_buffer, Frame::Data).await?;
        self.ack_next = false;
        // Some commands have different ack meechanism
        // ACK needs to be sent until there is a downlink
        self.uplink_cmds.retain(|cmd| {
            matches!(
                cmd,
                UplinkMacCommandCreator::RXParamSetupAns(_)
                    | UplinkMacCommandCreator::RXTimingSetupAns(_)
                    | UplinkMacCommandCreator::DlChannelAns(_)
                    | UplinkMacCommandCreator::TXParamSetupAns(_)
            )
        });
        // Handle received data
        if let Some(ref mut session_data) = self.session {
            // Parse payload and copy into user bufer is provided
            if let Some((_, rx_quality)) = rx_res {
                match parse_with_factory(radio_buffer, DefaultFactory) {
                    Ok(PhyPayload::Data(encoding::parser::DataPayload::Encrypted(encrypted))) => {
                        if session_data.devaddr() == &encrypted.fhdr().dev_addr() {
                            // clear all uplink cmds here after successfull downlink
                            self.uplink_cmds.clear();
                            self.adr_ack_cnt = 0;
                            let fcnt = encrypted.fhdr().fcnt() as u32;
                            // use temporary variable for ack_next to only confirm if the message was correctly handled
                            let ack_next = encrypted.is_confirmed();
                            if encrypted.validate_mic(session_data.newskey().inner(), fcnt)
                                && (fcnt > session_data.fcnt_down || fcnt == 0)
                            {
                                session_data.fcnt_down = fcnt;

                                let decrypted = encrypted
                                    .decrypt(
                                        Some(session_data.newskey().inner()),
                                        Some(session_data.appskey().inner()),
                                        session_data.fcnt_down,
                                    )
                                    .map_err(|_| crate::Error::<D>::Encoding)?;

                                //trace!("fhdr {:?}", decrypted.fhdr());
                                self.handle_downlink_macs(
                                    device,
                                    rx_quality,
                                    (&decrypted.fhdr()).into(),
                                )?;
                                let res = match decrypted.frm_payload() {
                                    FRMPayload::MACCommands(mac_cmds) => {
                                        self.handle_downlink_macs(
                                            device,
                                            rx_quality,
                                            (&mac_cmds).into(),
                                        )?;
                                        Ok(Some((0, rx_quality)))
                                    }
                                    FRMPayload::Data(rx_data) => {
                                        if let Some(rx) = rx {
                                            let to_copy = core::cmp::min(rx.len(), rx_data.len());
                                            rx[0..to_copy].copy_from_slice(&rx_data[0..to_copy]);
                                            Ok(Some((to_copy as u8, rx_quality)))
                                        } else {
                                            Ok(Some((0, rx_quality)))
                                        }
                                    }
                                    FRMPayload::None => Ok(Some((0, rx_quality))),
                                };
                                device
                                    .persist_to_non_volatile(&self.configuration, &self.credentials)
                                    .map_err(crate::device::Error::NonVolatileStore)?;
                                if res.is_ok() {
                                    self.ack_next = ack_next;
                                }
                                res
                            } else {
                                Err(crate::Error::Mac(crate::mac::Error::InvalidMic))
                            }
                        } else {
                            Err(crate::Error::Mac(crate::mac::Error::InvalidDevAddr))
                        }
                    }
                    Ok(_) => Err(crate::Error::Mac(crate::mac::Error::InvalidPayloadType)),
                    Err(_) => Err(crate::Error::Encoding),
                }
            } else if confirmed {
                Err(crate::Error::Mac(Error::NoResponse))
            } else {
                Ok(None)
            }
        } else {
            Err(crate::Error::Mac(Error::NetworkNotJoined))
        }
    }

    fn get_send_channels<D: Device>(
        &self,
        device: &mut D,
        frame: Frame,
    ) -> Result<[Option<<C as ChannelPlan<R>>::Channel>; NUM_OF_CHANNEL_BLOCKS], crate::Error<D>>
    {
        let mut channel_block_randoms = [0x00u32; NUM_OF_CHANNEL_BLOCKS];
        for channel_block_random in channel_block_randoms.iter_mut().take(NUM_OF_CHANNEL_BLOCKS) {
            *channel_block_random = device.rng().next_u32().map_err(crate::device::Error::Rng)?;
        }
        let mut channels = self
            .channel_plan
            .get_random_channels_from_blocks(channel_block_randoms)
            .unwrap_or([None, None, None, None, None, None, None, None, None, None]);

        // Place the preferred channel block first if a join request is being
        // executed, the index is greater than zero indicating a swap is needed, and
        // the index is valid.
        let swap_index = D::preferred_join_channel_block_index();
        if (frame == Frame::Join) && (swap_index > 0) && (swap_index < NUM_OF_CHANNEL_BLOCKS) {
            channels.swap(0, swap_index);
        }
        Ok(channels)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::device::DeviceSpecs;
    use crate::mac::region::channel_plan::dynamic::DynamicChannelPlan;
    use crate::mac::region::channel_plan::fixed::FixedChannelPlan;
    use crate::mac::region::eu868::EU868;
    use crate::mac::region::us915::US915;
    use crate::mac::Mac;

    struct DeviceMock;
    impl DeviceSpecs for DeviceMock {}

    #[test]
    fn validate_frequency() {
        assert!(Mac::<EU868, DynamicChannelPlan<EU868>>::validate_frequency::<DeviceMock>(
            863_000_000
        ));
        assert!(Mac::<EU868, DynamicChannelPlan<EU868>>::validate_frequency::<DeviceMock>(
            870_000_000
        ));
        assert!(!Mac::<EU868, DynamicChannelPlan<EU868>>::validate_frequency::<DeviceMock>(
            870_000_001
        ));
        assert!(Mac::<US915, FixedChannelPlan<US915>>::validate_frequency::<DeviceMock>(
            902_000_000
        ));
        assert!(Mac::<US915, FixedChannelPlan<US915>>::validate_frequency::<DeviceMock>(
            928_000_000
        ));
        assert!(!Mac::<US915, FixedChannelPlan<US915>>::validate_frequency::<DeviceMock>(
            928_000_001
        ));
    }

    #[test]
    fn validate_rx1_data_rate_offset() {
        assert!(
            Mac::<EU868, DynamicChannelPlan<EU868>>::validate_rx1_data_rate_offset::<DeviceMock>(0)
        );
        assert!(
            Mac::<EU868, DynamicChannelPlan<EU868>>::validate_rx1_data_rate_offset::<DeviceMock>(5)
        );
        assert!(!Mac::<EU868, DynamicChannelPlan<EU868>>::validate_rx1_data_rate_offset::<
            DeviceMock,
        >(6));
        assert!(
            Mac::<US915, DynamicChannelPlan<US915>>::validate_rx1_data_rate_offset::<DeviceMock>(0)
        );
        assert!(
            Mac::<US915, DynamicChannelPlan<US915>>::validate_rx1_data_rate_offset::<DeviceMock>(3)
        );
        assert!(!Mac::<US915, DynamicChannelPlan<US915>>::validate_rx1_data_rate_offset::<
            DeviceMock,
        >(4));
    }

    // #[test]
    // fn get_send_channels() {
    //     let mac_eu868 = Mac::<EU868, DynamicChannelPlan<EU868>>::new(
    //         Default::default(),
    //         Credentials::new([0u8; 8], [0u8; 8], [0u8; 16]),
    //     );
    //     let channels_eu868 = mac_eu868.get_send_channels(&mut DeviceMock, Frame::Join).unwrap();
    //     assert!(channels_eu868[0].is_some());
    //     assert!(channels_eu868[1].is_none());
    //     assert!(channels_eu868[2].is_none());
    //     assert!(channels_eu868[3].is_none());
    //     assert!(channels_eu868[4].is_none());
    //     assert!(channels_eu868[5].is_none());
    //     assert!(channels_eu868[6].is_none());
    //     assert!(channels_eu868[7].is_none());
    //     assert!(channels_eu868[8].is_none());
    //     assert!(channels_eu868[9].is_none());

    //     let mac_us915 = Mac::<US915, FixedChannelPlan<US915>>::new(
    //         Default::default(),
    //         Credentials::new([0u8; 8], [0u8; 8], [0u8; 16]),
    //     );
    //     let channels_us915 =
    //         mac_us915.get_send_channels(&mut DeviceMock::new(), Frame::Join).unwrap();
    //     assert!(channels_us915[0].is_some());
    //     assert!(channels_us915[1].is_some());
    //     assert!(channels_us915[2].is_some());
    //     assert!(channels_us915[3].is_some());
    //     assert!(channels_us915[4].is_some());
    //     assert!(channels_us915[5].is_some());
    //     assert!(channels_us915[6].is_some());
    //     assert!(channels_us915[7].is_some());
    //     assert!(channels_us915[8].is_some());
    //     assert!(channels_us915[9].is_none());
    // }
}
