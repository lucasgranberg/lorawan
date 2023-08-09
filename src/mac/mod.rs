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

use crate::{
    device::radio::{
        types::{RfConfig, TxConfig},
        Radio,
    },
    device::{
        radio::types::{CodingRate, RxQuality},
        radio_buffer::RadioBuffer,
        rng::Rng,
        timer::Timer,
        Device,
    },
    encoding::{
        creator::{DataPayloadCreator, JoinRequestCreator},
        default_crypto::DefaultFactory,
        maccommandcreator::{
            DevStatusAnsCreator, DlChannelAnsCreator, DutyCycleAnsCreator, LinkADRAnsCreator,
            NewChannelAnsCreator, RXParamSetupAnsCreator, RXTimingSetupAnsCreator,
            TXParamSetupAnsCreator, UplinkMacCommandCreator,
        },
        maccommands::{DLSettings, DownlinkMacCommand, MacCommandIterator, SerializableMacCommand},
        parser::{
            parse_with_factory, AsPhyPayloadBytes, DataHeader, DecryptedDataPayload,
            DecryptedJoinAcceptPayload, DevNonce, FCtrl, FRMPayload, PhyPayload, EUI64,
        },
    },
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

use futures::pin_mut;
use heapless::Vec;
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
    fn min_frequency<D: Device>() -> u32 {
        match D::min_frequency() {
            Some(device_min_frequency) => max(device_min_frequency, R::min_frequency()),
            None => R::min_frequency(),
        }
    }
    /// Get the maximum frequency, perhaps unique to the given end device
    fn max_frequency<D: Device>() -> u32 {
        match D::max_frequency() {
            Some(device_max_frequency) => min(device_max_frequency, R::max_frequency()),
            None => R::max_frequency(),
        }
    }
    /// Is the frequency within range for the given end device.
    fn validate_frequency<D: Device>(frequency: u32) -> bool {
        let frequency_range = Self::min_frequency::<D>()..=Self::max_frequency::<D>();
        frequency_range.contains(&frequency)
    }

    /// Get the maximum uplink data rate, perhaps unique to the given end device.
    fn max_data_rate<D: Device>() -> DR {
        match D::max_data_rate() {
            Some(device_max_data_rate) => {
                min(device_max_data_rate as u8, R::ul_data_rate_range().1 as u8).try_into().unwrap()
            }
            None => R::ul_data_rate_range().1,
        }
    }

    /// Get the minumum uplink data rate, perhaps unique to the given end device.
    fn min_data_rate<D: Device>() -> DR {
        match D::min_data_rate() {
            Some(device_min_data_rate) => {
                max(device_min_data_rate as u8, R::ul_data_rate_range().0 as u8).try_into().unwrap()
            }
            None => R::ul_data_rate_range().0,
        }
    }

    /// Is the RX1 data rate offset within range for the given end device?
    fn validate_rx1_data_rate_offset<D: Device>(rx1_dr_offset: u8) -> bool {
        R::get_rx1_dr(Self::min_data_rate::<D>(), rx1_dr_offset).is_ok()
            && R::get_rx1_dr(Self::max_data_rate::<D>(), rx1_dr_offset).is_ok()
    }

    /// Is the uplink data rate within range for the given end device?
    fn validate_data_rate<D: Device>(dr: u8) -> bool {
        DR::try_from(dr).unwrap().in_range((Self::min_data_rate::<D>(), Self::max_data_rate::<D>()))
    }

    /// Are the downlink data rate settings in range for the given end device?
    fn validate_dl_settings<D: Device>(dl_settings: DLSettings) -> (bool, bool) {
        let rx1_data_rate_offset_ack =
            Self::validate_rx1_data_rate_offset::<D>(dl_settings.rx1_dr_offset());
        let rx2_data_rate_ack = Self::validate_data_rate::<D>(dl_settings.rx2_data_rate());
        (rx1_data_rate_offset_ack, rx2_data_rate_ack)
    }

    /// Get the maximum EIRP for the end device.
    fn max_eirp<D: Device>() -> u8 {
        min(R::max_eirp(), D::max_eirp())
    }

    /// Get the transmission power based on the frame type.
    fn get_tx_pwr<D: Device>(frame: Frame, configuration: &Configuration) -> u8 {
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

        phy.set_app_eui(EUI64::new(self.credentials.app_eui).unwrap())
            .set_dev_eui(EUI64::new(self.credentials.dev_eui).unwrap())
            .set_dev_nonce(&devnonce.to_le_bytes());
        let vec = phy.build(&self.credentials.app_key).unwrap();

        buf.extend_from_slice(vec).unwrap();
    }

    fn get_rx_windows(&self, frame: Frame) -> RxWindows {
        match frame {
            Frame::Join => RxWindows {
                rx1_open: R::default_join_accept_delay1() - 15, // current observed duration to prepare to receive ranges from 0 to 13 ms ???
                rx2_open: R::default_join_accept_delay2() - 15,
            },
            Frame::Data => {
                let rx1_delay: u16 = self
                    .configuration
                    .rx_delay
                    .map(|delay| delay as u16 * 1000)
                    .unwrap_or(R::default_rx_delay());
                let rx2_delay = rx1_delay + 1000;
                RxWindows {
                    rx1_open: rx1_delay - 15, // current observed duration to prepare to receive ranges from 0 to 13 ms ???
                    rx2_open: rx2_delay - 15,
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

    fn handle_downlink_macs<D: Device>(
        &mut self,
        device: &mut D,
        rx_quality: RxQuality,
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
                            if device.adaptive_data_rate_enabled()
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
                    ans.set_margin(rx_quality.snr())?;
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

    async fn rx_with_timeout<'m, D: Device>(
        &self,
        frame: Frame,
        device: &mut D,
        radio_buffer: &'m mut RadioBuffer<256>,
        data_rate: DR,
        channel: &C::Channel,
    ) -> Result<Option<(usize, RxQuality)>, crate::Error<D>> {
        let windows = self.get_rx_windows(frame);

        let open_rx1_fut = device
            .timer()
            .at(windows.get_open(&Window::_1) as u64)
            .map_err(crate::device::Error::Timer)?;
        let open_rx2_fut = device
            .timer()
            .at(windows.get_open(&Window::_2) as u64)
            .map_err(crate::device::Error::Timer)?;
        pin_mut!(open_rx2_fut);
        open_rx1_fut.await;

        {
            radio_buffer.clear();
            let rf_config = self.create_rf_config(&frame, &Window::_1, data_rate, channel)?;
            trace!("rf config {:?}", rf_config);
            match device.radio().rx(rf_config, 1, radio_buffer.as_raw_slice()).await {
                Ok(ret) => {
                    return Ok(Some(ret));
                }
                // Bail on error other than timeout ???
                Err(_e) => {}
            }
        }

        open_rx2_fut.await;

        {
            radio_buffer.clear();
            let rf_config = self.create_rf_config(&frame, &Window::_2, data_rate, channel)?;
            trace!("rf config {:?}", rf_config);
            match device.radio().rx(rf_config, 1, radio_buffer.as_raw_slice()).await {
                Ok(ret) => Ok(Some(ret)),
                Err(e) => Err(crate::Error::Device(crate::device::Error::Radio(e))),
            }
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
            let packet = phy.build(data, &dyn_cmds, session.newskey(), session.appskey())?;
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
    ) -> Result<Option<(usize, RxQuality)>, crate::Error<D>> {
        let tx_buffer = radio_buffer.clone();

        for _ in 0..self.configuration.number_of_transmissions {
            let channels = self.get_send_channels(device, frame);
            for channel in channels {
                if let Some(chn) = channel {
                    let tx_data_rate = R::override_ul_data_rate_if_necessary(
                        self.tx_data_rate(),
                        frame,
                        chn.get_ul_frequency(),
                    );
                    let tx_config = self.create_tx_config(frame, &chn, tx_data_rate)?;
                    trace!("tx config {:?}", tx_config);
                    let _ms = device
                        .radio()
                        .tx(tx_config, tx_buffer.as_ref())
                        .await
                        .map_err(crate::device::Error::Radio)?;
                    device.timer().reset();

                    match self
                        .rx_with_timeout(frame, device, radio_buffer, tx_data_rate, &chn)
                        .await
                    {
                        Ok(Some((num_read, rx_quality))) => {
                            radio_buffer.inc(num_read);
                            return Ok(Some((num_read, rx_quality)));
                        }
                        Ok(None) => {}
                        Err(_e) => {}
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
    pub async fn join<'m, D: Device>(
        &'m mut self,
        device: &'m mut D,
        radio_buffer: &'m mut RadioBuffer<256>,
    ) -> Result<(), crate::Error<D>> {
        self.credentials.incr_dev_nonce();
        device
            .persist_to_non_volatile(&self.configuration, &self.credentials)
            .map_err(crate::device::Error::NonVolatileStore)?;
        self.create_join_request(radio_buffer);
        let rx_res = self.send_buffer(device, radio_buffer, Frame::Join).await?;
        if rx_res.is_some() {
            match parse_with_factory(radio_buffer.as_mut(), DefaultFactory)? {
                PhyPayload::JoinAccept(encrypted) => {
                    let decrypt = DecryptedJoinAcceptPayload::new_from_encrypted(
                        encrypted,
                        &self.credentials.app_key,
                    );
                    if decrypt.validate_mic(&self.credentials.app_key) {
                        let session = Session::derive_new(
                            &decrypt,
                            DevNonce::<[u8; 2]>::new(self.credentials.dev_nonce.to_le_bytes())
                                .unwrap(),
                            &self.credentials,
                        );
                        trace!("msg {=[u8]:02X}", decrypt.as_bytes());
                        trace!("new {=[u8]:02X}", session.newskey().0);
                        trace!("app {=[u8]:02X}", session.appskey().0);
                        trace!("rx1 {:?}", decrypt.dl_settings().rx1_dr_offset());
                        trace!("rx2 {:?}", decrypt.dl_settings().rx2_data_rate());
                        trace!("rx2 {:?}", decrypt.c_f_list());
                        self.session.replace(session);
                        self.adr_ack_cnt = 0;

                        let (rx1_data_rate_offset_ack, rx2_data_rate_ack) =
                            Self::validate_dl_settings::<D>(decrypt.dl_settings());
                        trace!("{}{}", rx1_data_rate_offset_ack, rx2_data_rate_ack);
                        if rx1_data_rate_offset_ack && rx2_data_rate_ack {
                            self.handle_dl_settings(decrypt.dl_settings())?
                        }

                        let delay = match decrypt.rx_delay() {
                            0 => 1,
                            _ => decrypt.rx_delay(),
                        };
                        self.configuration.rx_delay = Some(delay);
                        if let Some(cf_list) = decrypt.c_f_list() {
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
        confirmed: bool,
        rx: Option<&mut [u8]>,
    ) -> Result<Option<(usize, RxQuality)>, crate::Error<D>> {
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
                    Ok(PhyPayload::Data(encrypted_data)) => {
                        if session_data.devaddr() == &encrypted_data.fhdr().dev_addr() {
                            // clear all uplink cmds here after successfull downlink
                            self.uplink_cmds.clear();
                            self.adr_ack_cnt = 0;
                            let fcnt = encrypted_data.fhdr().fcnt() as u32;
                            // use temporary variable for ack_next to only confirm if the message was correctly handled
                            let ack_next = encrypted_data.is_confirmed();
                            if encrypted_data.validate_mic(session_data.newskey(), fcnt)
                                && (fcnt > session_data.fcnt_down || fcnt == 0)
                            {
                                session_data.fcnt_down = fcnt;

                                // * the decrypt will always work when we have verified MIC previously
                                let decrypted = DecryptedDataPayload::new_from_encrypted(
                                    encrypted_data,
                                    Some(session_data.newskey()),
                                    Some(session_data.appskey()),
                                    session_data.fcnt_down,
                                )
                                .unwrap();

                                trace!("fhdr {:?}", decrypted.fhdr());
                                self.handle_downlink_macs(
                                    device,
                                    rx_quality,
                                    (&decrypted.fhdr()).into(),
                                )?;
                                let res = match decrypted.frm_payload()? {
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
                                            Ok(Some((to_copy, rx_quality)))
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
                    Err(e) => Err(crate::Error::Encoding(e)),
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
    ) -> [Option<<C as ChannelPlan<R>>::Channel>; NUM_OF_CHANNEL_BLOCKS] {
        let mut channel_block_randoms = [0x00u32; NUM_OF_CHANNEL_BLOCKS];
        for channel_block_random in channel_block_randoms.iter_mut().take(NUM_OF_CHANNEL_BLOCKS) {
            *channel_block_random = device.rng().next_u32().unwrap_or(1);
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
        channels
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::mac::region::channel_plan::dynamic::DynamicChannelPlan;
    use crate::mac::region::channel_plan::fixed::FixedChannelPlan;
    use crate::mac::region::eu868::EU868;
    use crate::mac::region::us915::US915;
    use crate::mac::Mac;
    use crate::tests::*;

    use super::types::{Credentials, Frame};

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
    fn get_send_channels() {
        let mac_eu868 = Mac::<EU868, DynamicChannelPlan<EU868>>::new(
            Default::default(),
            Credentials::new([0u8; 8], [0u8; 8], [0u8; 16]),
        );
        let channels_eu868 = mac_eu868.get_send_channels(&mut DeviceMock::new(), Frame::Join);
        assert!(channels_eu868[0].is_some());
        assert!(channels_eu868[1].is_none());
        assert!(channels_eu868[2].is_none());
        assert!(channels_eu868[3].is_none());
        assert!(channels_eu868[4].is_none());
        assert!(channels_eu868[5].is_none());
        assert!(channels_eu868[6].is_none());
        assert!(channels_eu868[7].is_none());
        assert!(channels_eu868[8].is_none());
        assert!(channels_eu868[9].is_none());

        let mac_us915 = Mac::<US915, FixedChannelPlan<US915>>::new(
            Default::default(),
            Credentials::new([0u8; 8], [0u8; 8], [0u8; 16]),
        );
        let channels_us915 = mac_us915.get_send_channels(&mut DeviceMock::new(), Frame::Join);
        assert!(channels_us915[0].is_some());
        assert!(channels_us915[1].is_some());
        assert!(channels_us915[2].is_some());
        assert!(channels_us915[3].is_some());
        assert!(channels_us915[4].is_some());
        assert!(channels_us915[5].is_some());
        assert!(channels_us915[6].is_some());
        assert!(channels_us915[7].is_some());
        assert!(channels_us915[8].is_some());
        assert!(channels_us915[9].is_none());
    }
}
