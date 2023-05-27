use core::fmt::Debug;

//pub mod mac_1_0_4;
pub mod region;
pub mod types;

use core::{
    cmp::{max, min},
    marker::PhantomData,
};

use self::region::{
    channel_plan::{Channel, ChannelPlan},
    Region,
};
use crate::{
    device::radio::{
        types::{RfConfig, TxConfig},
        Radio,
    },
    device::{
        non_volatile_store::NonVolatileStore,
        radio::types::{CodingRate, RxQuality},
        radio_buffer::RadioBuffer,
        rng::Rng,
        timer::Timer,
        Device, DeviceSpecs,
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
use futures::{future::select, future::Either, pin_mut};
use heapless::Vec;
use types::*;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

pub trait MacDevice<R, S>: Device
where
    R: Region,
    S: DeviceSpecs,
{
    fn persist_to_non_volatile(
        &mut self,
        configuration: &Configuration,
        credentials: &Credentials,
    ) -> Result<(), <<Self as Device>::NonVolatileStore as NonVolatileStore>::Error> {
        let storable = Storable {
            rx1_data_rate_offset: configuration.rx1_data_rate_offset,
            rx_delay: configuration.rx_delay,
            rx2_data_rate: configuration.rx2_data_rate,
            rx2_frequency: configuration.rx2_frequency,
            dev_nonce: credentials.dev_nonce,
        };
        if let Ok(old_storable) = self.non_volatile_store().load() {
            if storable != old_storable {
                trace!("overwrite {} {}", old_storable, storable);
                self.non_volatile_store().save(storable)?;
            } else {
                trace!("nothing changed {}", storable);
            }
        } else {
            trace!("Save fresh {}", storable);
            self.non_volatile_store().save(storable)?;
        }
        Ok(())
    }
    fn hydrate_from_non_volatile(
        non_volatile_store: &mut Self::NonVolatileStore,
        app_eui: [u8; 8],
        dev_eui: [u8; 8],
        app_key: [u8; 16],
    ) -> Result<
        (Configuration, Credentials),
        <<Self as Device>::NonVolatileStore as NonVolatileStore>::Error,
    > {
        let storable: Storable = non_volatile_store.load()?;
        let configuration = Configuration {
            rx1_data_rate_offset: storable.rx1_data_rate_offset,
            rx_delay: storable.rx_delay,
            rx2_data_rate: storable.rx2_data_rate,
            rx2_frequency: storable.rx2_frequency,
            ..Default::default()
        };
        let mut credentials = Credentials::new(app_eui, dev_eui, app_key);
        credentials.dev_nonce = storable.dev_nonce;
        Ok((configuration, credentials))
    }
    fn get_max_eirp(&self) -> u8 {
        min(R::max_eirp(), Self::max_eirp())
    }
    fn get_tx_pwr(&mut self, frame: Frame, configuration: &Configuration) -> u8 {
        match frame {
            Frame::Join => self.get_max_eirp(),
            Frame::Data => configuration.tx_power.unwrap_or(self.get_max_eirp()),
        }
    }
    fn max_data_rate(&self) -> DR {
        match S::max_data_rate() {
            Some(device_max_data_rate) => min(device_max_data_rate as u8, R::max_data_rate() as u8)
                .try_into()
                .unwrap(),
            None => R::max_data_rate(),
        }
    }
    fn min_data_rate(&self) -> DR {
        match S::min_data_rate() {
            Some(device_min_data_rate) => max(device_min_data_rate as u8, R::min_data_rate() as u8)
                .try_into()
                .unwrap(),
            None => R::min_data_rate(),
        }
    }
    fn max_frequency(&self) -> u32 {
        match S::max_frequency() {
            Some(device_max_frequency) => min(device_max_frequency, R::max_frequency()),
            None => R::max_frequency(),
        }
    }
    fn min_frequency(&self) -> u32 {
        match S::min_frequency() {
            Some(device_min_frequency) => max(device_min_frequency, R::min_frequency()),
            None => R::min_frequency(),
        }
    }
    fn validate_frequency(&self, frequency: u32) -> bool {
        let frequency_range = self.min_frequency()..self.max_frequency();
        frequency_range.contains(&frequency)
    }
    fn validate_rx1_data_rate_offset(&mut self, rx1_dr_offset: u8) -> bool {
        (0u8..=5u8).contains(&rx1_dr_offset)
    }
    fn validate_data_rate(&self, dr: u8) -> bool {
        ((self.min_data_rate() as u8)..=(self.max_data_rate() as u8)).contains(&dr)
    }
    fn validate_dl_settings(&mut self, dl_settings: DLSettings) -> (bool, bool) {
        let rx1_data_rate_offset_ack =
            self.validate_rx1_data_rate_offset(dl_settings.rx1_dr_offset());
        let rx2_data_rate_ack = self.validate_data_rate(dl_settings.rx2_data_rate());
        (rx1_data_rate_offset_ack, rx2_data_rate_ack)
    }
}

pub struct Mac<R, S, C>
where
    R: Region,
    S: DeviceSpecs,
    C: ChannelPlan<R> + Default,
{
    pub(crate) session: Option<Session>,
    pub(crate) channel_plan: C,
    pub(crate) region: PhantomData<R>,
    pub(crate) device: PhantomData<S>,
    pub(crate) uplink_cmds: Vec<UplinkMacCommandCreator, 15>,
    pub(crate) ack_next: bool,
    pub(crate) configuration: Configuration,
    pub(crate) credentials: Credentials,
}

impl<R, S, C> Mac<R, S, C>
where
    R: region::Region,
    S: DeviceSpecs,
    C: ChannelPlan<R> + Default,
{
    pub fn new(configuration: Configuration, credentials: Credentials) -> Self {
        Self {
            session: None,
            channel_plan: Default::default(),
            region: PhantomData,
            device: PhantomData,
            uplink_cmds: Vec::new(),
            ack_next: false,
            configuration,
            credentials,
        }
    }
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
        self.configuration
            .tx_data_rate
            .unwrap_or(R::default_data_rate())
    }
    fn rx1_data_rate_offset(&self) -> u8 {
        self.configuration
            .rx1_data_rate_offset
            .unwrap_or(R::default_rx1_data_rate_offset())
    }
    fn rx1_data_rate(&self, tx_dr: DR) -> DR {
        let offset = self.rx1_data_rate_offset();
        if offset < tx_dr as u8 {
            (tx_dr as u8 - offset).try_into().unwrap_or(tx_dr)
        } else {
            DR::_0
        }
    }
    fn rx2_data_rate(&self, frame: &Frame) -> DR {
        match frame {
            Frame::Join => R::default_rx2_data_rate(),
            Frame::Data => self
                .configuration
                .rx2_data_rate
                .unwrap_or(R::default_rx2_data_rate()),
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
                rx1_open: R::default_join_accept_delay1() - 150,
                rx1_close: R::default_join_accept_delay2() - 200,
                rx2_open: R::default_join_accept_delay2() - 150,
                rx2_close: R::default_join_accept_delay2() + 3000,
            },
            Frame::Data => {
                let rx1_delay: u16 = self
                    .configuration
                    .rx_delay
                    .map(|delay| delay as u16 * 1000)
                    .unwrap_or(R::default_rx_delay());
                let rx2_delay = rx1_delay + 1000;
                RxWindows {
                    rx1_open: rx1_delay - 150,
                    rx1_close: rx2_delay - 200,
                    rx2_open: rx2_delay - 150,
                    rx2_close: rx2_delay + 3000,
                }
            }
        }
    }
    fn create_tx_config<D>(
        &self,
        device: &mut D,
        frame: Frame,
        channel: &C::Channel,
    ) -> Result<TxConfig, crate::Error<D>>
    where
        D: MacDevice<R, S>,
    {
        let pw = device.get_tx_pwr(frame, &self.configuration);
        let data_rate = R::convert_data_rate(self.tx_data_rate())?;
        let tx_config = TxConfig {
            pw,
            rf: RfConfig {
                frequency: channel.get_frequency().value(),
                coding_rate: CodingRate::_4_5,
                data_rate,
            },
        };
        Ok(tx_config)
    }
    fn create_rf_config<D>(
        &self,
        frame: &Frame,
        window: &Window,
        data_rate: DR,
        channel: &C::Channel,
    ) -> Result<RfConfig, crate::Error<D>>
    where
        D: MacDevice<R, S>,
    {
        let data_rate = match window {
            Window::_1 => self.rx1_data_rate(data_rate),
            Window::_2 => self.rx2_data_rate(frame),
        };
        let data_rate = R::convert_data_rate(data_rate)?;
        let rf_config = match (frame, window) {
            (Frame::Join, Window::_1) => RfConfig {
                frequency: channel.get_frequency().value(),
                coding_rate: CodingRate::_4_5,
                data_rate,
            },
            (Frame::Join, Window::_2) => RfConfig {
                frequency: R::default_rx2_frequency(),
                coding_rate: CodingRate::_4_5,
                data_rate,
            },
            (Frame::Data, Window::_1) => RfConfig {
                frequency: channel.get_frequency().value(),
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
    fn handle_downlink_macs<D>(
        &mut self,
        device: &mut D,
        rx_quality: RxQuality,
        cmds: MacCommandIterator<'_, DownlinkMacCommand<'_>>,
    ) -> Result<(), crate::Error<D>>
    where
        D: MacDevice<R, S>,
    {
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
                    Some(UplinkMacCommandCreator::DutyCycleAns(
                        DutyCycleAnsCreator::new(),
                    ))
                }
                DownlinkMacCommand::RXParamSetupReq(payload) => {
                    let mut ans = RXParamSetupAnsCreator::new();
                    let (mut rx1_data_rate_offset_ack, mut rx2_data_rate_ack) =
                        device.validate_dl_settings(payload.dl_settings());
                    let channel_ack = self
                        .channel_plan
                        .validate_frequency(payload.frequency().value())
                        .is_ok();
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
                    if payload.channel_index() < R::default_channels() {
                        None //silently ignore if default channel
                    } else {
                        let data_rate_range_ack = device
                            .validate_data_rate(payload.data_rate_range().min_data_range())
                            && device.validate_data_rate(payload.data_rate_range().max_data_rate())
                            && payload.data_rate_range().min_data_range()
                                < payload.data_rate_range().max_data_rate();

                        let channel_frequency_ack = payload.frequency().value() == 0
                            || device.validate_frequency(payload.frequency().value());

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
                        device.validate_frequency(payload.frequency().value());
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
                    Some(UplinkMacCommandCreator::RXTimingSetupAns(
                        RXTimingSetupAnsCreator::new(),
                    ))
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
                self.uplink_cmds
                    .push(uplink_cmd)
                    .map_err(|_| crate::mac::Error::FOptsFull)?
            }
        }
        Ok(())
    }

    async fn rx_with_timeout<'m, D>(
        &self,
        frame: Frame,
        device: &mut D,
        radio_buffer: &'m mut RadioBuffer<256>,
        data_rate: DR,
        channel: &C::Channel,
    ) -> Result<Option<(usize, RxQuality)>, crate::Error<D>>
    where
        D: MacDevice<R, S>,
    {
        let windows = self.get_rx_windows(frame);
        let mut window = Window::_1;

        loop {
            radio_buffer.clear();
            let rf_config = self.create_rf_config(&frame, &window, data_rate, channel)?;
            trace!("rf config {:?}", rf_config);
            let open_fut = device
                .timer()
                .at(windows.get_open(&window) as u64)
                .map_err(crate::device::Error::Timer)?;
            let timeout_fut = device
                .timer()
                .at(windows.get_close(&window) as u64)
                .map_err(crate::device::Error::Timer)?;
            let rx_fut = device.radio().rx(rf_config, radio_buffer.as_raw_slice());
            pin_mut!(rx_fut);
            pin_mut!(timeout_fut);
            open_fut.await;

            // Wait until either RX is complete or if we've reached window close
            match select(rx_fut, timeout_fut).await {
                // RX is complete!
                Either::Left((r, close_at)) => match r {
                    Ok(ret) => {
                        return Ok(Some(ret));
                    }
                    // Ignore errors or timeouts and wait until the RX2 window is ready.
                    Err(e) => {
                        if let Window::_1 = window {
                            window = Window::_2;
                            close_at.await;
                        } else {
                            return Err(crate::Error::Device(crate::device::Error::Radio(e)));
                        }
                    }
                },
                // Timeout! Jumpt to next window.
                Either::Right(_) => {
                    if let Window::_1 = window {
                        window = Window::_2;
                    } else {
                        return Ok(None);
                    }
                }
            }
        }
    }

    fn prepare_buffer<D>(
        &mut self,
        data: &[u8],
        fport: u8,
        confirmed: bool,
        radio_buffer: &mut RadioBuffer<256>,
        device: &D,
    ) -> Result<u32, crate::Error<D>>
    where
        D: MacDevice<R, S>,
    {
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
            radio_buffer
                .extend_from_slice(packet)
                .map_err(crate::device::Error::RadioBuffer)?;
            Ok(fcnt)
        } else {
            Err(crate::Error::Mac(crate::mac::Error::NetworkNotJoined))
        }
    }
    async fn send_buffer<D>(
        &self,
        device: &mut D,
        radio_buffer: &mut RadioBuffer<256>,
        frame: Frame,
    ) -> Result<Option<(usize, RxQuality)>, crate::Error<D>>
    where
        D: MacDevice<R, S>,
    {
        for _ in 0..self.configuration.number_of_transmissions {
            let random = device.rng().next_u32().map_err(crate::device::Error::Rng)?;
            let tx_data_rate = self.tx_data_rate();
            let channel = self
                .channel_plan
                .get_random_channel(random, frame, tx_data_rate)?;

            let tx_config = self.create_tx_config(device, frame, &channel)?;
            trace!("tx config {:?}", tx_config);
            // Transmit the join payload
            let _ms = device
                .radio()
                .tx(tx_config, radio_buffer.as_ref())
                .await
                .map_err(crate::device::Error::Radio)?;
            device.timer().reset();

            // Receive join response within RX window
            let rx_res = self
                .rx_with_timeout(frame, device, radio_buffer, tx_data_rate, &channel)
                .await?;
            if let Some((num_read, _)) = rx_res {
                radio_buffer.inc(num_read);
            }
            if rx_res.is_some() {
                return Ok(rx_res);
            }
        }
        Ok(None)
    }
    pub async fn join<'m, D>(
        &'m mut self,
        device: &'m mut D,
        radio_buffer: &'m mut RadioBuffer<256>,
    ) -> Result<(), crate::Error<D>>
    where
        D: MacDevice<R, S>,
    {
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

                        let (rx1_data_rate_offset_ack, rx2_data_rate_ack) =
                            device.validate_dl_settings(decrypt.dl_settings());
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
    pub async fn send<D>(
        &mut self,
        device: &mut D,
        radio_buffer: &mut RadioBuffer<256>,
        data: &[u8],
        fport: u8,
        confirmed: bool,
        rx: Option<&mut [u8]>,
    ) -> Result<Option<(usize, RxQuality)>, crate::Error<D>>
    where
        D: MacDevice<R, S>,
    {
        if let Some(ref mut session_data) = self.session {
            if !session_data.is_expired() {
                session_data.fcnt_up_increment();
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
}
