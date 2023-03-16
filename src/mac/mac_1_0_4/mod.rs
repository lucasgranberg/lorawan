use core::{
    cmp::{max, min},
    fmt::Debug,
    future::Future,
    marker::PhantomData,
};

use self::{
    encoding::{
        creator::{DataPayloadCreator, JoinRequestCreator},
        maccommandcreator::*,
        maccommands::DownlinkMacCommand,
        parser::{DecryptedDataPayload, DecryptedJoinAcceptPayload},
    },
    region::{
        channel_plan::{Channel, ChannelPlan},
        Region,
    },
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
        default_crypto::DefaultFactory,
        keys::{CryptoFactory, AES128},
        maccommandcreator::{
            DevStatusAnsCreator, DlChannelAnsCreator, DutyCycleAnsCreator, LinkADRAnsCreator,
            NewChannelAnsCreator, RXParamSetupAnsCreator, RXTimingSetupAnsCreator,
            TXParamSetupAnsCreator,
        },
        maccommands::{MacCommandIterator, SerializableMacCommand},
        parser::{
            parse_with_factory, DataHeader, DevAddr, DevNonce, FCtrl, FRMPayload, PhyPayload, EUI64,
        },
    },
    Error, Frame, Window, DR,
};
use futures::{future::select, future::Either, pin_mut};
use generic_array::{typenum::U256, GenericArray};
use heapless::Vec;

use super::RxWindows;
mod encoding;
pub mod region;
pub struct Session {
    newskey: AES128,
    appskey: AES128,
    devaddr: DevAddr<[u8; 4]>,
    fcnt_up: u32,
    fcnt_down: u32,
}
impl Session {
    pub fn derive_new<T: AsRef<[u8]> + AsMut<[u8]>, F: CryptoFactory>(
        decrypt: &DecryptedJoinAcceptPayload<T, F>,
        devnonce: DevNonce<[u8; 2]>,
        credentials: &Credentials,
    ) -> Self {
        Self::new(
            decrypt.derive_newskey(&devnonce, &credentials.app_key),
            decrypt.derive_appskey(&devnonce, &credentials.app_key),
            DevAddr::new([
                decrypt.dev_addr().as_ref()[0],
                decrypt.dev_addr().as_ref()[1],
                decrypt.dev_addr().as_ref()[2],
                decrypt.dev_addr().as_ref()[3],
            ])
            .unwrap(),
        )
    }

    pub fn new(newskey: AES128, appskey: AES128, devaddr: DevAddr<[u8; 4]>) -> Self {
        Self {
            newskey,
            appskey,
            devaddr,
            fcnt_up: 0,
            fcnt_down: 0,
        }
    }

    pub fn newskey(&self) -> &AES128 {
        &self.newskey
    }

    pub fn appskey(&self) -> &AES128 {
        &self.appskey
    }

    pub fn devaddr(&self) -> &DevAddr<[u8; 4]> {
        &self.devaddr
    }

    pub fn fcnt_up(&self) -> u32 {
        self.fcnt_up
    }

    pub fn fcnt_up_increment(&mut self) {
        self.fcnt_up += 1;
    }
}
pub struct Credentials {
    pub app_eui: [u8; 8],
    pub dev_eui: [u8; 8],
    pub app_key: AES128,
    pub dev_nonce: u16,
}
pub struct Status<C, R>
where
    R: Region,
    C: ChannelPlan<R> + Default,
{
    pub(crate) confirm_next: bool,
    pub(crate) max_duty_cycle: f32,
    pub(crate) tx_power: Option<i8>,
    pub(crate) tx_data_rate: Option<DR>,
    pub(crate) rx1_data_rate_offset: Option<DR>,
    pub(crate) rx1_delay: Option<u8>,
    pub(crate) rx2_data_rate: Option<DR>,
    pub(crate) rx_quality: Option<RxQuality>,
    pub(crate) battery_level: Option<f32>,
    pub(crate) channel_plan: C,
    pub(crate) number_of_transmissions: u8,
    region: PhantomData<R>,
}
impl<C, R> Default for Status<C, R>
where
    R: Region,
    C: ChannelPlan<R> + Default,
{
    fn default() -> Self {
        Self {
            tx_data_rate: None,
            confirm_next: false,
            tx_power: None,
            max_duty_cycle: 0.0,
            rx1_data_rate_offset: None,
            rx1_delay: None,
            rx2_data_rate: None,
            rx_quality: None,
            battery_level: None,
            number_of_transmissions: 0,
            channel_plan: Default::default(),
            region: Default::default(),
        }
    }
}
pub struct Mac<'a, R, D, C>
where
    R: Region,
    D: Device,
    C: ChannelPlan<R> + Default,
{
    credentials: &'a mut Credentials,
    session: &'a mut Option<Session>,
    status: &'a mut Status<C, R>,
    region: PhantomData<R>,
    device: PhantomData<D>,
    cmds: Vec<UplinkMacCommandCreator, 15>,
}
impl<'a, R, D, C> Mac<'a, R, D, C>
where
    R: region::Region,
    D: Device + Debug,
    C: ChannelPlan<R> + Default,
{
    pub fn new(
        credentials: &'a mut Credentials,
        session: &'a mut Option<Session>,
        status: &'a mut Status<C, R>,
    ) -> Self {
        Self {
            credentials,
            session,
            status,
            region: PhantomData::default(),
            device: PhantomData::default(),
            cmds: Vec::new(),
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

    fn get_rx_windows(&self, frame: Frame) -> super::RxWindows {
        match frame {
            Frame::Join => RxWindows {
                rx1_open: 1000,
                rx1_close: 1900,
                rx2_open: 2000,
                rx2_close: 2900,
            },
            Frame::Data => RxWindows {
                rx1_open: 1000,
                rx1_close: 1900,
                rx2_open: 2000,
                rx2_close: 2900,
            },
        }
    }
    fn get_max_eirp(&self) -> i8 {
        min(R::max_eirp(), D::max_eirp())
    }
    fn get_tx_pwr(&self, frame: Frame) -> i8 {
        match frame {
            Frame::Join => self.get_max_eirp(),
            Frame::Data => self.status.tx_power.unwrap_or(R::max_eirp()),
        }
    }
    fn max_data_rate(&self) -> DR {
        match D::max_data_rate() {
            Some(device_max_data_rate) => min(device_max_data_rate as u8, R::max_data_rate() as u8)
                .try_into()
                .unwrap(),
            None => R::max_data_rate(),
        }
    }
    fn min_data_rate(&self) -> DR {
        match D::min_data_rate() {
            Some(device_min_data_rate) => max(device_min_data_rate as u8, R::min_data_rate() as u8)
                .try_into()
                .unwrap(),
            None => R::max_data_rate(),
        }
    }
    fn max_frequency(&self) -> u32 {
        match D::max_frequency() {
            Some(device_max_frequency) => min(device_max_frequency, R::max_frequency()),
            None => R::max_frequency(),
        }
    }
    fn min_frequency(&self) -> u32 {
        match D::min_frequency() {
            Some(device_min_frequency) => max(device_min_frequency, R::min_frequency()),
            None => R::min_frequency(),
        }
    }
    fn create_tx_config(
        &self,
        frame: Frame,
        data_rate: DR,
        channel: &C::Channel,
    ) -> Result<TxConfig, Error<D>> {
        let pw = self.get_tx_pwr(frame);
        let data_rate = R::convert_data_rate(data_rate).map_err(Error::Region)?;
        let tx_config = match frame {
            Frame::Join => TxConfig {
                pw,
                rf: RfConfig {
                    frequency: channel.get_frequency().value(),
                    coding_rate: CodingRate::_4_5,
                    data_rate,
                },
            },
            Frame::Data => TxConfig {
                pw,
                rf: RfConfig {
                    frequency: channel.get_frequency().value(),
                    coding_rate: CodingRate::_4_5,
                    data_rate,
                },
            },
        };
        Ok(tx_config)
    }
    fn create_rf_config(
        &self,
        frame: &Frame,
        window: &Window,
        data_rate: DR,
        channel: &C::Channel,
    ) -> Result<RfConfig, Error<D>> {
        let data_rate = match (frame, window) {
            (Frame::Data, Window::_2) => R::convert_data_rate(
                self.status
                    .rx2_data_rate
                    .unwrap_or(R::default_rx2_data_rate()),
            ),
            _ => R::convert_data_rate(data_rate),
        }
        .map_err(Error::Region)?;
        let rf_config = match (frame, window) {
            (Frame::Join, Window::_1) => RfConfig {
                frequency: channel.get_frequency().value(),
                coding_rate: CodingRate::_4_5,
                data_rate,
            },
            (Frame::Join, Window::_2) => RfConfig {
                frequency: channel.get_frequency().value(),
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
    fn handle_downlink_macs(
        &mut self,
        device: &mut D,
        cmds: MacCommandIterator<'_, DownlinkMacCommand<'_>>,
    ) -> Result<(), Error<D>> {
        self.cmds.clear();
        let mut channel_mask = self.status.channel_plan.get_channel_mask();
        let mut cmd_iter = cmds.into_iter().peekable();
        while let Some(cmd) = cmd_iter.next() {
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
                    let new_tx_power =
                        R::modify_dbm(payload.tx_power(), self.status.tx_power, R::max_eirp());
                    let new_data_rate: Result<Option<DR>, ()> = if payload.data_rate() == 0xF {
                        Ok(self.status.tx_data_rate)
                    } else {
                        DR::try_from(payload.data_rate()).map(Some)
                    };
                    ans.set_tx_power_ack(new_tx_power.is_ok());
                    ans.set_data_rate_ack(new_data_rate.is_ok());

                    let channel_mask_res = self.status.channel_plan.handle_channel_mask(
                        &mut channel_mask,
                        payload.channel_mask(),
                        payload.redundancy().channel_mask_control(),
                    );
                    ans.set_channel_mask_ack(channel_mask_res.is_ok());
                    // check if next command is also a LinkADRReq, if not process the atomic block
                    match cmd_iter.peek() {
                        Some(DownlinkMacCommand::LinkADRReq(_)) => (),
                        _ => {
                            match (new_tx_power, new_data_rate, channel_mask_res) {
                                (Ok(new_tx_power), Ok(new_data_rate), Ok(_)) => {
                                    if self
                                        .status
                                        .channel_plan
                                        .set_channel_mask(channel_mask)
                                        .is_ok()
                                    {
                                        self.status.tx_power = new_tx_power;
                                        self.status.tx_data_rate = new_data_rate;
                                        self.status.number_of_transmissions =
                                            payload.redundancy().number_of_transmissions();
                                        ans.set_channel_mask_ack(true);
                                    } else {
                                        ans.set_channel_mask_ack(false);
                                    }
                                }
                                _ => {
                                    ans.set_channel_mask_ack(false);
                                }
                            }
                            //reset channel mask to match actual status
                            channel_mask = self.status.channel_plan.get_channel_mask();
                        }
                    }

                    Some(UplinkMacCommandCreator::LinkADRAns(ans))
                }
                DownlinkMacCommand::DutyCycleReq(payload) => {
                    self.status.max_duty_cycle = payload.max_duty_cycle();
                    Some(UplinkMacCommandCreator::DutyCycleAns(
                        DutyCycleAnsCreator::new(),
                    ))
                }
                DownlinkMacCommand::RXParamSetupReq(payload) => {
                    let mut ans = RXParamSetupAnsCreator::new();
                    let rx1_data_rate_offset_res: Result<DR, _> =
                        payload.dl_settings().rx1_dr_offset().try_into();
                    let mut rx1_data_rate_offset_ack = false;
                    if let Ok(rx1_dr_offset) = rx1_data_rate_offset_res {
                        if rx1_dr_offset as u8 <= 5 {
                            rx1_data_rate_offset_ack = true;
                        }
                    }

                    let rx2_data_rate_res: Result<DR, _> =
                        payload.dl_settings().rx2_data_rate().try_into();
                    let mut rx2_data_rate_ack = false;
                    if let Ok(rx2_data_rate) = rx2_data_rate_res {
                        if ((self.min_data_rate() as u8)..(self.max_data_rate() as u8))
                            .contains(&(rx2_data_rate as u8))
                        {
                            rx2_data_rate_ack = true;
                        }
                    }
                    ans.set_rx1_data_rate_offset_ack(rx1_data_rate_offset_ack);
                    ans.set_rx2_data_rate_ack(rx2_data_rate_ack);
                    if rx1_data_rate_offset_ack && rx2_data_rate_ack {
                        self.status.rx1_data_rate_offset = Some(rx1_data_rate_offset_res.unwrap());
                        self.status.rx2_data_rate = Some(rx2_data_rate_res.unwrap());
                    }
                    Some(UplinkMacCommandCreator::RXParamSetupAns(ans))
                }
                DownlinkMacCommand::DevStatusReq(_) => {
                    let mut ans = DevStatusAnsCreator::new();
                    match self.status.battery_level {
                        Some(battery_level) => ans.set_battery((battery_level * 253.0) as u8 + 1),
                        None => ans.set_battery(255),
                    };
                    match self.status.rx_quality {
                        Some(rx_quality) => ans.set_margin(rx_quality.snr()).unwrap(),
                        None => ans.set_margin(0).unwrap(),
                    };
                    Some(UplinkMacCommandCreator::DevStatusAns(ans))
                }
                DownlinkMacCommand::NewChannelReq(payload) => {
                    if payload.channel_index() < R::default_channels() {
                        None //silently ignore if default channel
                    } else {
                        let data_rate_range =
                            self.min_data_rate() as u8..self.max_data_rate() as u8;
                        let data_rate_range_ack = data_rate_range
                            .contains(&payload.data_rate_range().min_data_range())
                            && data_rate_range.contains(&payload.data_rate_range().max_data_rate());

                        let frequency_range = self.min_frequency()..self.max_frequency();
                        let channel_frequency_ack = frequency_range
                            .contains(&payload.frequency().value())
                            || payload.frequency().value() == 0;

                        let mut ans = NewChannelAnsCreator::new();
                        ans.set_channel_frequency_ack(channel_frequency_ack);
                        ans.set_data_rate_range_ack(data_rate_range_ack);
                        if data_rate_range_ack && channel_frequency_ack {
                            match self.status.channel_plan.handle_new_channel_req(payload) {
                                Ok(_) => ans.set_channel_frequency_ack(true),
                                Err(_) => ans.set_channel_frequency_ack(false),
                            };
                        }
                        Some(UplinkMacCommandCreator::NewChannelAns(ans))
                    }
                }
                DownlinkMacCommand::DlChannelReq(payload) => {
                    let mut ans = DlChannelAnsCreator::new();
                    let frequency_range = self.min_frequency()..self.max_frequency();
                    let mut channel_frequency_ack =
                        frequency_range.contains(&payload.frequency().value());
                    //let mut uplink_frequency_exists_ack = false;
                    let uplink_frequency_exists_ack = self
                        .status
                        .channel_plan
                        .check_uplink_frequency_exists(payload.channel_index() as usize);
                    if channel_frequency_ack {
                        channel_frequency_ack = self
                            .status
                            .channel_plan
                            .handle_dl_channel_req(payload)
                            .is_ok()
                    }
                    ans.set_uplink_frequency_exists_ack(uplink_frequency_exists_ack);
                    ans.set_channel_frequency_ack(channel_frequency_ack);
                    Some(UplinkMacCommandCreator::DlChannelAns(ans))
                }
                DownlinkMacCommand::RXTimingSetupReq(payload) => {
                    self.status.rx1_delay = Some(payload.delay());
                    Some(UplinkMacCommandCreator::RXTimingSetupAns(
                        RXTimingSetupAnsCreator::new(),
                    ))
                }
                DownlinkMacCommand::TXParamSetupReq(_) => {
                    if R::supports_tx_param_setup() {
                        let ans = TXParamSetupAnsCreator::new();
                        let _ret = Some(UplinkMacCommandCreator::TXParamSetupAns(ans));
                        todo!("TXParamSetupReq not implemented yet");
                    } else {
                        None
                    }
                }
            };
            if let Some(uplink_cmd) = res {
                self.add_uplink_cmd(uplink_cmd)?
            }
        }
        Ok(())
    }
    fn add_uplink_cmd(&mut self, cmd: UplinkMacCommandCreator) -> Result<(), Error<D>> {
        self.cmds
            .push(cmd)
            .map_err(|_| Error::Mac(crate::mac::Error::FOptsFull))
    }

    async fn rx_with_timeout<'m>(
        &self,
        frame: Frame,
        device: &mut D,
        radio_buffer: &'m mut RadioBuffer<256>,
        data_rate: DR,
        channel: &C::Channel,
    ) -> Result<Option<(usize, RxQuality)>, Error<D>> {
        let windows = self.get_rx_windows(frame);
        let mut window = Window::_1;

        radio_buffer.clear();

        loop {
            let rf_config = self.create_rf_config(&frame, &window, data_rate, channel)?;
            device.timer().reset();
            device
                .timer()
                .at(windows.get_open(&window) as u64)
                .await
                .map_err(|e| Error::Device(crate::device::Error::Timer(e)))?;
            let timeout_fut = device.timer().at(windows.get_close(&window) as u64);
            let rx_fut = device.radio().rx(rf_config, radio_buffer.as_mut());
            pin_mut!(rx_fut);
            pin_mut!(timeout_fut);

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
                            close_at
                                .await
                                .map_err(|e| Error::Device(crate::device::Error::Timer(e)))?;
                        } else {
                            return Err(Error::Device(crate::device::Error::Radio(e)));
                        }
                    }
                },
                // Timeout! Jumpt to next window.
                Either::Right(_) => {
                    if let Window::_1 = window {
                        window = Window::_2;
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(None)
    }

    fn prepare_buffer<CRYPTO: CryptoFactory>(
        &mut self,
        data: &[u8],
        fport: u8,
        confirmed: bool,
        radio_buffer: &mut RadioBuffer<256>,
        factory: CRYPTO,
    ) -> Result<u32, Error<D>> {
        if let Some(session) = self.session {
            // check if FCnt is used up
            if session.fcnt_up() == (0xFFFF + 1) {
                // signal that the session is expired
                return Err(Error::Mac(crate::mac::Error::SessionExpired));
            }
            let fcnt = session.fcnt_up();
            let mut phy: DataPayloadCreator<GenericArray<u8, U256>, CRYPTO> =
                DataPayloadCreator::new(GenericArray::default(), factory);

            let mut fctrl = FCtrl(0x0, true);

            if self.status.confirm_next {
                fctrl.set_ack();
                self.status.confirm_next = false;
            }

            phy.set_confirmed(confirmed)
                .set_uplink(true)
                .set_fctrl(&fctrl)
                .set_f_port(fport)
                .set_dev_addr(*session.devaddr())
                .set_fcnt(fcnt);

            let mut dyn_cmds: Vec<&dyn SerializableMacCommand, 8> = Vec::new();
            for cmd in self.cmds.iter() {
                if let Err(_e) = dyn_cmds.push(cmd) {
                    panic!("dyn_cmds too small compared to cmds")
                }
            }
            let packet = phy
                .build(data, &dyn_cmds, session.newskey(), session.appskey())
                .map_err(|e| Error::Encoding(e))?;

            radio_buffer.clear();
            radio_buffer.extend_from_slice(packet).unwrap();
            Ok(fcnt)
        } else {
            Err(Error::Mac(crate::mac::Error::NetworkNotJoined))
        }
    }
}

impl<'a, R, D, C> crate::mac::Mac<R, D> for Mac<'a, R, D, C>
where
    R: region::Region + 'static,
    D: Device + Debug,
    C: ChannelPlan<R> + Default + 'a,
{
    type Error = Error<D>;
    type JoinFuture<'m> = impl Future<Output = Result<(), Self::Error>> + 'm where Self: 'm;

    fn join<'m>(
        &'m mut self,
        device: &'m mut D,
        radio_buffer: &'m mut RadioBuffer<256>,
    ) -> Self::JoinFuture<'m> {
        self.create_join_request(radio_buffer);

        async move {
            let random = device
                .rng()
                .next_u32()
                .map_err(|e| Error::Device(crate::device::Error::Rng(e)))?;
            let channel = self
                .status
                .channel_plan
                .get_random_channel(random, DR::_0)
                .map_err(|_| Error::Mac(crate::mac::Error::NoValidChannelFound))?;

            let tx_config = self.create_tx_config(Frame::Join, R::default_data_rate(), &channel)?;
            // Transmit the join payload
            let _ms = device
                .radio()
                .tx(tx_config, radio_buffer.as_ref())
                .await
                .map_err(|e| Error::Device(crate::device::Error::Radio(e)))?;
            //device.timer().reset();

            // Receive join response within RX window
            self.rx_with_timeout(Frame::Join, device, radio_buffer, DR::_0, &channel)
                .await?;

            match parse_with_factory(radio_buffer.as_mut(), DefaultFactory)
                .map_err(Error::Encoding)?
            {
                PhyPayload::JoinAccept(encrypted) => {
                    let decrypt = DecryptedJoinAcceptPayload::new_from_encrypted(
                        encrypted,
                        &self.credentials.app_key,
                    );
                    if decrypt.validate_mic(&self.credentials.app_key) {
                        let session = Session::derive_new(
                            &decrypt,
                            DevNonce::<[u8; 2]>::new_from_raw(
                                self.credentials.dev_nonce.to_le_bytes(),
                            ),
                            self.credentials,
                        );
                        self.session.replace(session);
                        Ok(())
                    } else {
                        Err(Error::Mac(crate::mac::Error::InvalidMic))
                    }
                }
                _ => Err(Error::Mac(crate::mac::Error::InvalidPayloadType)),
            }
        }
    }

    type SendFuture<'m> = impl Future<Output = Result<usize, Self::Error>> + 'm where Self: 'm, D: 'm;
    fn send<'m>(
        &'m mut self,
        device: &'m mut D,
        radio_buffer: &'m mut RadioBuffer<256>,
        data: &'m [u8],
        fport: u8,
        confirmed: bool,
        rx: Option<&'m mut [u8]>,
    ) -> Self::SendFuture<'m> {
        async move {
            if self.session.is_none() {
                return Err(Error::Mac(crate::mac::Error::NetworkNotJoined));
            }
            let random = device
                .rng()
                .next_u32()
                .map_err(|e| Error::Device(crate::device::Error::Rng(e)))?;
            let data_rate = self.status.tx_data_rate.unwrap_or(R::default_data_rate());
            let channel = self
                .status
                .channel_plan
                .get_random_channel(random, data_rate)
                .map_err(|_| Error::Mac(crate::mac::Error::NoValidChannelFound))?;
            //self.mac.handle_uplink_macs(macs);
            // Prepare transmission buffer
            let _ = self.prepare_buffer(data, fport, confirmed, radio_buffer, DefaultFactory)?;

            // Send data
            let tx_config = self.create_tx_config(Frame::Data, data_rate, &channel)?;
            // Transmit our data packet
            let _ms = device
                .radio()
                .tx(tx_config, radio_buffer.as_ref())
                .await
                .map_err(|e| Error::Device(crate::device::Error::Radio(e)))?;

            // Wait for received data within window
            device.timer().reset();
            let rx_res = self
                .rx_with_timeout(Frame::Data, device, radio_buffer, data_rate, &channel)
                .await?;
            if let Some((_len, rx_quality)) = rx_res {
                self.status.rx_quality = Some(rx_quality);
            }
            // Handle received data
            if let Some(ref mut session_data) = self.session {
                // Parse payload and copy into user bufer is provided
                let res = parse_with_factory(radio_buffer.as_mut(), DefaultFactory);
                match res {
                    Ok(PhyPayload::Data(encrypted_data)) => {
                        if session_data.devaddr() == &encrypted_data.fhdr().dev_addr() {
                            let fcnt = encrypted_data.fhdr().fcnt() as u32;
                            let confirmed = encrypted_data.is_confirmed();
                            if encrypted_data.validate_mic(session_data.newskey(), fcnt)
                                && (fcnt > session_data.fcnt_down || fcnt == 0)
                            {
                                session_data.fcnt_down = fcnt;
                                // increment the FcntUp since we have received
                                // downlink - only reason to not increment
                                // is if confirmed frame is sent and no
                                // confirmation (ie: downlink) occurs
                                session_data.fcnt_up_increment();

                                // * the decrypt will always work when we have verified MIC previously
                                let decrypted = DecryptedDataPayload::new_from_encrypted(
                                    encrypted_data,
                                    Some(session_data.newskey()),
                                    Some(session_data.appskey()),
                                    session_data.fcnt_down,
                                )
                                .unwrap();

                                self.cmds.clear(); //clear cmd buffer
                                self.handle_downlink_macs(device, (&decrypted.fhdr()).into())?;

                                if confirmed {
                                    self.status.confirm_next = true;
                                }
                                match decrypted.frm_payload().map_err(Error::Encoding)? {
                                    FRMPayload::MACCommands(mac_cmds) => {
                                        self.handle_downlink_macs(device, (&mac_cmds).into())?;
                                        Ok(0)
                                    }
                                    FRMPayload::Data(rx_data) => {
                                        if let Some(rx) = rx {
                                            let to_copy = core::cmp::min(rx.len(), rx_data.len());
                                            rx[0..to_copy].copy_from_slice(&rx_data[0..to_copy]);
                                            Ok(to_copy)
                                        } else {
                                            Ok(0)
                                        }
                                    }
                                    FRMPayload::None => Ok(0),
                                }
                            } else {
                                Err(Error::Mac(crate::mac::Error::InvalidMic))
                            }
                        } else {
                            Err(Error::Mac(crate::mac::Error::InvalidDevAddr))
                        }
                    }
                    Ok(_) => Err(Error::Mac(crate::mac::Error::InvalidPayloadType)),
                    Err(e) => Err(Error::Encoding(e)),
                }
            } else {
                Err(Error::Mac(crate::mac::Error::NetworkNotJoined))
            }
        }
    }
}
