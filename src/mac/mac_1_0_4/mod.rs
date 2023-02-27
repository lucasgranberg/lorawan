use core::{future::Future, marker::PhantomData};

use self::encoding::{
    creator::DataPayloadCreator,
    maccommandcreator::UplinkMacCommandCreator,
    maccommands::DownlinkMacCommand,
    parser::{DecryptedDataPayload, DecryptedJoinAcceptPayload},
};
use crate::{
    encoding::{
        default_crypto,
        keys::{CryptoFactory, AES128},
        maccommands::{MacCommandIterator, SerializableMacCommand},
        parser::{
            parse_with_factory, DataHeader, DevAddr, DevNonce, FCtrl, FRMPayload, PhyPayload,
        },
        securityhelpers,
    },
    radio::{
        types::{RadioBuffer, RfConfig, TxConfig},
        PhyRxTx,
    },
    timer::Timer,
    CfList, Error, Frame, Window, DR,
};
use futures::{future::select, future::Either, pin_mut};
use generic_array::{typenum::U256, GenericArray};
use heapless::Vec;
use rand_core::RngCore;

use super::{MType, Region, Timings};
mod encoding;
mod region;
struct Session {
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
struct Credentials {
    join_eui: [u8; 8],
    dev_eui: [u8; 8],
    app_key: AES128,
    dev_nonce: u16,
}
struct Status {
    tx_dr: Option<DR>,
    rx_data_rates: Option<(DR, DR)>,
    cf_list: Option<CfList>,
    confirm_next: bool,
}
struct Mac<'a, R, C, T, P, RNG: RngCore, const N: usize = 256> {
    credentials: &'a mut Credentials,
    session: &'a mut Option<Session>,
    status: &'a mut Status,
    timer: T,
    rng: RNG,
    region: PhantomData<R>,
    crypto: PhantomData<C>,
    radio: PhantomData<P>,
    cmds: Vec<UplinkMacCommandCreator, 15>,
}
impl<'a, R, C, T, P, RNG> Mac<'a, R, C, T, P, RNG>
where
    C: CryptoFactory + Default,
    T: Timer,
    P: crate::radio::PhyRxTx,
    RNG: RngCore,
{
    fn create_join_request(&mut self, radio_buffer: &mut RadioBuffer<256>) {
        radio_buffer.clear();
        let buf = radio_buffer.as_mut();
        buf[0] = MType::JoinRequest as u8;
        buf[1..9].copy_from_slice(self.credentials.join_eui.as_slice());
        buf[9..17].copy_from_slice(self.credentials.dev_eui.as_slice());
        buf[17..19].copy_from_slice(self.credentials.dev_nonce.to_le_bytes().as_slice());

        let len = buf.len();
        let mic = securityhelpers::calculate_mic(
            &buf[..len - 4],
            default_crypto::DefaultFactory.new_mac(&self.credentials.app_key),
        );
        buf[len - 4..].copy_from_slice(&mic.0[..]);
    }

    fn get_timings(&self, frame: Frame) -> super::Timings {
        match frame {
            Frame::Join => Timings {
                rx1_open: 1000,
                rx1_close: 1900,
                rx2_open: 2000,
                rx2_close: 2900,
            },
            Frame::Data => Timings {
                rx1_open: 1000,
                rx1_close: 1900,
                rx2_open: 2000,
                rx2_close: 2900,
            },
        }
    }

    fn create_rf_config(&self) -> RfConfig {
        todo!()
    }
    fn create_tx_config(&self, frame: &Frame) -> TxConfig {
        todo!()
    }
    fn handle_downlink_macs<'b, 'c>(
        &mut self,
        cmds: MacCommandIterator<'b, DownlinkMacCommand<'c>>,
    ) {
        todo!()
    }
    fn add_uplink_cmd(&mut self, cmd: UplinkMacCommandCreator) -> Result<(), Error<P::PhyError>> {
        self.cmds.push(cmd).map_err(|_| Error::FOptsFull)
    }

    async fn rx_with_timeout<'m>(
        &mut self,
        frame: Frame,
        radio: &mut P,
        radio_buffer: &'m mut RadioBuffer<256>,
    ) -> Result<usize, Error<P::PhyError>> {
        let timings = self.get_timings(frame);
        let mut window = Window::_1;

        radio_buffer.clear();

        loop {
            let rf_config = self.create_rf_config();
            self.timer.at(timings.get_open(&window) as u64).await;
            let timeout_fut = self.timer.at(timings.get_close(&window) as u64);
            let rx_fut = radio.rx(rf_config, radio_buffer.as_mut());
            pin_mut!(rx_fut);
            pin_mut!(timeout_fut);
            // Wait until either RX is complete or if we've reached window close
            match select(rx_fut, timeout_fut).await {
                // RX is complete!
                Either::Left((r, close_at)) => match r {
                    Ok((sz, _q)) => {
                        return Ok(sz);
                    }
                    // Ignore errors or timeouts and wait until the RX2 window is ready.
                    _ => close_at.await,
                },
                // Timeout! Jumpt to next window.
                Either::Right(_) => (),
            }

            if let Window::_1 = window {
                window = Window::_2;
            } else {
                break;
            }
        }
        todo!()
    }

    fn prepare_buffer(
        &mut self,
        data: &[u8],
        fport: u8,
        confirmed: bool,
        radio_buffer: &mut RadioBuffer<256>,
    ) -> Result<u32, Error<P::PhyError>> {
        if let Some(session) = self.session {
            // check if FCnt is used up
            if session.fcnt_up() == (0xFFFF + 1) {
                // signal that the session is expired
                return Err(Error::SessionExpired);
            }
            let fcnt = session.fcnt_up();
            let mut phy: DataPayloadCreator<GenericArray<u8, U256>, C> =
                DataPayloadCreator::default();

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

impl<'a, R, P, C, T, RNG> crate::mac::Mac<R, P, T> for Mac<'a, R, C, T, P, RNG>
where
    R: Region,
    P: PhyRxTx,
    C: CryptoFactory + Default,
    T: Timer,
    RNG: RngCore,
{
    type Error = Error<P::PhyError>;
    type JoinFuture<'m> = impl Future<Output = Result<(), Self::Error>> + 'm where Self: 'm, P: 'm;

    fn join<'m>(
        &'m mut self,
        radio: &'m mut P,
        radio_buffer: &'m mut RadioBuffer<256>,
    ) -> Self::JoinFuture<'m> {
        self.create_join_request(radio_buffer);

        let tx_config = R::create_tx_config(
            MType::JoinRequest,
            self.status.tx_dr.unwrap_or(R::get_default_datarate()),
        );
        async move {
            // Transmit the join payload
            let _ms = radio
                .tx(tx_config, radio_buffer.as_ref())
                .await
                .map_err(Error::Radio)?;
            self.timer.reset();

            // Receive join response within RX window
            self.rx_with_timeout(Frame::Join, radio, radio_buffer)
                .await?;

            match parse_with_factory(radio_buffer.as_mut(), C::default()) {
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

    type SendFuture<'m> = impl Future<Output = Result<usize, Self::Error>> + 'm where Self: 'm, P: 'm;
    fn send<'m>(
        &'m mut self,
        radio: &'m mut P,
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
            let _ = self.prepare_buffer(data, fport, confirmed, radio_buffer)?;

            // Send data
            let tx_config = self.create_tx_config(&Frame::Data);
            // Transmit our data packet
            let _ms = radio
                .tx(tx_config, radio_buffer.as_ref())
                .await
                .map_err(Error::Radio)?;

            // Wait for received data within window
            self.timer.reset();
            self.rx_with_timeout(Frame::Data, radio, radio_buffer)
                .await?;

            // Handle received data
            if let Some(ref mut session_data) = self.session {
                // Parse payload and copy into user bufer is provided
                let res = parse_with_factory(radio_buffer.as_mut(), C::default());
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
