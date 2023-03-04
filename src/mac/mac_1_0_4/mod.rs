use core::{future::Future, marker::PhantomData};

use self::encoding::{
    creator::{DataPayloadCreator, JoinRequestCreator},
    maccommandcreator::*,
    maccommands::DownlinkMacCommand,
    parser::{DecryptedDataPayload, DecryptedJoinAcceptPayload},
};
use crate::{
    channel_mask::ChannelMask,
    device::radio::{
        types::{RadioBuffer, RfConfig, TxConfig},
        PhyRxTx,
    },
    device::{radio::types::RxQuality, timer::Timer, Device},
    encoding::{
        default_crypto::DefaultFactory,
        keys::{CryptoFactory, AES128},
        maccommandcreator::{
            DevStatusAnsCreator, DutyCycleAnsCreator, LinkADRAnsCreator, NewChannelAnsCreator,
            RXParamSetupAnsCreator,
        },
        maccommands::{MacCommandIterator, SerializableMacCommand},
        parser::{
            parse_with_factory, DataHeader, DevAddr, DevNonce, FCtrl, FRMPayload, PhyPayload, EUI64,
        },
    },
    CfList, Error, Frame, Window, DR,
};
use futures::{future::select, future::Either, pin_mut};
use generic_array::{typenum::U256, GenericArray};
use heapless::Vec;
use rand_core::RngCore;

use super::{Region, RxWindows};
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
pub struct Status {
    pub(crate) tx_data_rate: Option<DR>,
    pub(crate) cf_list: Option<CfList>,
    pub(crate) confirm_next: bool,
    pub(crate) channel_mask: ChannelMask,
    pub(crate) tx_power: Option<i8>,
    pub(crate) max_duty_cycle: f32,
    pub(crate) rx1_dr_offset: Option<u8>,
    pub(crate) rx2_data_rate: Option<u8>,
    pub(crate) rx_quality: Option<RxQuality>,
    pub(crate) battery_level: Option<f32>,
}
impl Default for Status {
    fn default() -> Self {
        Status {
            tx_data_rate: None,
            cf_list: None,
            confirm_next: false,
            channel_mask: ChannelMask::new_from_raw(&[0xFF, 0xFF]),
            tx_power: None,
            max_duty_cycle: 0.0,
            rx1_dr_offset: None,
            rx2_data_rate: None,
            rx_quality: None,
            battery_level: None,
        }
    }
}
pub struct Mac<'a, R, D>
where
    R: Region,
    D: Device,
{
    credentials: &'a mut Credentials,
    session: &'a mut Option<Session>,
    status: &'a mut Status,
    region: PhantomData<R>,
    device: PhantomData<D>,
    cmds: Vec<UplinkMacCommandCreator, 15>,
}
impl<'a, R, D> Mac<'a, R, D>
where
    R: Region,
    D: Device,
{
    pub fn new(
        credentials: &'a mut Credentials,
        session: &'a mut Option<Session>,
        status: &'a mut Status,
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
    fn get_tx_pwr(&self) -> i8 {
        R::max_eirp()
    }

    fn create_rf_config(&self, frame: Frame, device: &mut D) -> RfConfig {
        match frame {
            Frame::Join => R::create_rf_config(frame, device.rng().next_u32(), None),
            Frame::Data => todo!(),
        }
    }
    fn create_tx_config(&self, frame: Frame, device: &mut D) -> TxConfig {
        TxConfig {
            pw: self.get_tx_pwr(),
            rf: self.create_rf_config(frame, device),
        }
    }
    fn handle_downlink_macs<'b>(
        &mut self,
        cmds: MacCommandIterator<'_, DownlinkMacCommand<'b>>,
    ) -> Result<(), Error> {
        self.cmds.clear();
        for cmd in cmds {
            match cmd {
                DownlinkMacCommand::LinkCheckAns(_) => todo!(),
                DownlinkMacCommand::LinkADRReq(payload) => {
                    let mut ans = LinkADRAnsCreator::new();
                    let new_tx_power =
                        R::modify_dbm(payload.tx_power(), self.status.tx_power, R::max_eirp());
                    let new_data_rate = match payload.data_rate() {
                        0 => Ok(Some(DR::_0)),
                        1 => Ok(Some(DR::_1)),
                        2 => Ok(Some(DR::_2)),
                        3 => Ok(Some(DR::_3)),
                        4 => Ok(Some(DR::_4)),
                        5 => Ok(Some(DR::_5)),
                        6 => Ok(Some(DR::_6)),
                        7 => Ok(Some(DR::_7)),
                        8 => Ok(Some(DR::_8)),
                        9 => Ok(Some(DR::_9)),
                        10 => Ok(Some(DR::_10)),
                        11 => Ok(Some(DR::_11)),
                        12 => Ok(Some(DR::_12)),
                        13 => Ok(Some(DR::_13)),
                        14 => Ok(Some(DR::_14)),
                        //The value 0xF (decimal 15) of either DataRate or TXPower means that the end-device SHALL ignore that field and keep the current parameter values.
                        //15 => Ok(Some(DR::_15)),
                        15 => Ok(self.status.tx_data_rate),
                        _ => Err(()),
                    };
                    ans.set_tx_power_ack(new_tx_power.is_ok());
                    ans.set_data_rate_ack(new_data_rate.is_ok());
                    ans.set_channel_mask_ack(true);
                    if new_tx_power.is_ok() && new_data_rate.is_ok() {
                        self.status.tx_power = new_tx_power.unwrap();
                        self.status.tx_data_rate = new_data_rate.unwrap();
                        self.status.channel_mask = payload.channel_mask();
                    }
                    self.add_uplink_cmd(UplinkMacCommandCreator::LinkADRAns(ans))?;
                }
                DownlinkMacCommand::DutyCycleReq(payload) => {
                    self.status.max_duty_cycle = payload.max_duty_cycle();
                    self.add_uplink_cmd(UplinkMacCommandCreator::DutyCycleAns(
                        DutyCycleAnsCreator::new(),
                    ))?;
                }
                DownlinkMacCommand::RXParamSetupReq(payload) => {
                    let mut ans = RXParamSetupAnsCreator::new();
                    self.status.rx1_dr_offset = Some(payload.dl_settings().rx1_dr_offset());
                    self.status.rx2_data_rate = Some(payload.dl_settings().rx2_data_rate());
                    ans.set_rx1_data_rate_offset_ack(true);
                    ans.set_rx2_data_rate_ack(true);
                    self.add_uplink_cmd(UplinkMacCommandCreator::RXParamSetupAns(ans))?;
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
                    self.add_uplink_cmd(UplinkMacCommandCreator::DevStatusAns(ans))?;
                }
                DownlinkMacCommand::NewChannelReq(_) => todo!(),
                DownlinkMacCommand::RXTimingSetupReq(_) => todo!(),
                DownlinkMacCommand::TXParamSetupReq(_) => todo!(),
                DownlinkMacCommand::DlChannelReq(_) => todo!(),
                DownlinkMacCommand::DeviceTimeAns(_) => todo!(),
            }
        }
        Ok(())
    }
    fn add_uplink_cmd(&mut self, cmd: UplinkMacCommandCreator) -> Result<(), Error> {
        self.cmds.push(cmd).map_err(|_| Error::FOptsFull)
    }

    async fn rx_with_timeout<'m>(
        &mut self,
        frame: Frame,
        device: &mut D,
        radio_buffer: &'m mut RadioBuffer<256>,
    ) -> Result<Option<(usize, RxQuality)>, <<D as Device>::PhyRxTx as PhyRxTx>::PhyError> {
        let windows = self.get_rx_windows(frame);
        let mut window = Window::_1;

        radio_buffer.clear();

        loop {
            let rf_config = self.create_rf_config(frame, device);
            device.timer().reset();
            device.timer().at(windows.get_open(&window) as u64).await;
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
                            close_at.await
                        } else {
                            return Err(e);
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

    fn prepare_buffer<C: CryptoFactory>(
        &mut self,
        data: &[u8],
        fport: u8,
        confirmed: bool,
        radio_buffer: &mut RadioBuffer<256>,
        factory: C,
    ) -> Result<u32, Error> {
        if let Some(session) = self.session {
            // check if FCnt is used up
            if session.fcnt_up() == (0xFFFF + 1) {
                // signal that the session is expired
                return Err(Error::SessionExpired);
            }
            let fcnt = session.fcnt_up();
            let mut phy: DataPayloadCreator<GenericArray<u8, U256>, C> =
                DataPayloadCreator::new(GenericArray::default(), factory);

            let mut fctrl = FCtrl(0x0, true);
            if self.status.confirm_next {
                fctrl.set_ack();
                self.status.confirm_next = false;
            }

            phy.set_confirmed(confirmed)
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

            match phy.build(data, &dyn_cmds, session.newskey(), session.appskey()) {
                Ok(packet) => {
                    radio_buffer.clear();
                    radio_buffer.extend_from_slice(packet).unwrap();
                    Ok(fcnt)
                }
                Err(e) => Err(Error::UnableToPreparePayload),
            }
        } else {
            Err(Error::NetworkNotJoined)
        }
    }
}

impl<'a, R, D> crate::mac::Mac<R, D> for Mac<'a, R, D>
where
    R: Region,
    D: Device,
{
    type Error = Error;
    type JoinFuture<'m> = impl Future<Output = Result<(), Self::Error>> + 'm where Self: 'm;

    fn join<'m>(
        &'m mut self,
        device: &'m mut D,
        radio_buffer: &'m mut RadioBuffer<256>,
    ) -> Self::JoinFuture<'m> {
        self.create_join_request(radio_buffer);

        let tx_config = self.create_tx_config(Frame::Join, device);
        async move {
            // Transmit the join payload
            let _ms = device
                .radio()
                .tx(tx_config, radio_buffer.as_ref())
                .await
                .map_err(|e| Error::Device(crate::device::Error::Radio))?;
            //device.timer().reset();

            // Receive join response within RX window
            self.rx_with_timeout(Frame::Join, device, radio_buffer)
                .await
                .map_err(|e| Self::Error::Device(crate::device::Error::Radio))?;

            match parse_with_factory(radio_buffer.as_mut(), DefaultFactory) {
                Ok(PhyPayload::JoinAccept(encrypted)) => {
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
                        Err(Error::InvalidMic)
                    }
                }
                _ => Err(Error::UnableToDecodePayload("")),
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
                return Err(Error::NetworkNotJoined);
            }
            //self.mac.handle_uplink_macs(macs);
            // Prepare transmission buffer
            let _ = self.prepare_buffer(data, fport, confirmed, radio_buffer, DefaultFactory)?;

            // Send data
            let tx_config = self.create_tx_config(Frame::Data, device);
            // Transmit our data packet
            let _ms = device
                .radio()
                .tx(tx_config, radio_buffer.as_ref())
                .await
                .map_err(|e| Error::Device(crate::device::Error::Radio))?;

            // Wait for received data within window
            device.timer().reset();
            let rx_res = self
                .rx_with_timeout(Frame::Data, device, radio_buffer)
                .await
                .map_err(|e| Error::Device(crate::device::Error::Radio))?;
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
                                self.handle_downlink_macs((&decrypted.fhdr()).into());

                                if confirmed {
                                    self.status.confirm_next = true;
                                }
                                match decrypted.frm_payload() {
                                    Ok(FRMPayload::MACCommands(mac_cmds)) => {
                                        self.handle_downlink_macs((&mac_cmds).into());
                                        Ok(0)
                                    }
                                    Ok(FRMPayload::Data(rx_data)) => {
                                        if let Some(rx) = rx {
                                            let to_copy = core::cmp::min(rx.len(), rx_data.len());
                                            rx[0..to_copy].copy_from_slice(&rx_data[0..to_copy]);
                                            Ok(to_copy)
                                        } else {
                                            Ok(0)
                                        }
                                    }
                                    Ok(FRMPayload::None) => Ok(0),
                                    Err(_) => Err(Error::UnableToDecodePayload("")),
                                }
                            } else {
                                Err(Error::InvalidMic)
                            }
                        } else {
                            Err(Error::InvalidDevAddr)
                        }
                    }
                    Ok(_) => Err(Error::UnableToDecodePayload("")),
                    Err(e) => Err(Error::UnableToDecodePayload(e)),
                }
            } else {
                Err(Error::NetworkNotJoined)
            }
        }
    }
}
