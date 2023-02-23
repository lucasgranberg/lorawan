// Copyright (c) 2018-2020 Ivaylo Petrov
//
// Licensed under the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//
// Author: Ivaylo Petrov <ivajloip@gmail.com>

use super::maccommands::*;
use crate::encoding::{maccommandcreator::*, maccommands::SerializableMacCommand};

#[derive(Debug, PartialEq, Eq)]
pub enum MacCommandCreator {
    LinkCheckReq(LinkCheckReqCreator),
    LinkCheckAns(LinkCheckAnsCreator),
    LinkADRReq(LinkADRReqCreator),
    LinkADRAns(LinkADRAnsCreator),
    DutyCycleReq(DutyCycleReqCreator),
    DutyCycleAns(DutyCycleAnsCreator),
    RXParamSetupReq(RXParamSetupReqCreator),
    RXParamSetupAns(RXParamSetupAnsCreator),
    DevStatusReq(DevStatusReqCreator),
    DevStatusAns(DevStatusAnsCreator),
    NewChannelReq(NewChannelReqCreator),
    NewChannelAns(NewChannelAnsCreator),
    RXTimingSetupReq(RXTimingSetupReqCreator),
    RXTimingSetupAns(RXTimingSetupAnsCreator),
    TXParamSetupReq(TXParamSetupReqCreator),
    TXParamSetupAns(TXParamSetupAnsCreator),
    DlChannelReq(DlChannelReqCreator),
    DlChannelAns(DlChannelAnsCreator),
    DeviceTimeReq(DeviceTimeReqCreator),
    DeviceTimeAns(DeviceTimeAnsCreator),
}

impl MacCommandCreator {
    #![allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match *self {
            Self::LinkCheckReq(_) => LinkCheckReqPayload::len(),
            Self::LinkCheckAns(_) => LinkCheckAnsPayload::len(),
            Self::LinkADRReq(_) => LinkADRReqPayload::len(),
            Self::LinkADRAns(_) => LinkADRAnsPayload::len(),
            Self::DutyCycleReq(_) => DutyCycleReqPayload::len(),
            Self::DutyCycleAns(_) => DutyCycleAnsPayload::len(),
            Self::RXParamSetupReq(_) => RXParamSetupReqPayload::len(),
            Self::RXParamSetupAns(_) => RXParamSetupAnsPayload::len(),
            Self::DevStatusReq(_) => DevStatusReqPayload::len(),
            Self::DevStatusAns(_) => DevStatusAnsPayload::len(),
            Self::NewChannelReq(_) => NewChannelReqPayload::len(),
            Self::NewChannelAns(_) => NewChannelAnsPayload::len(),
            Self::RXTimingSetupReq(_) => RXTimingSetupReqPayload::len(),
            Self::RXTimingSetupAns(_) => RXTimingSetupAnsPayload::len(),
            Self::TXParamSetupReq(_) => TXParamSetupReqPayload::len(),
            Self::TXParamSetupAns(_) => TXParamSetupAnsPayload::len(),
            Self::DlChannelReq(_) => DlChannelReqPayload::len(),
            Self::DlChannelAns(_) => DlChannelAnsPayload::len(),
            Self::DeviceTimeReq(_) => DeviceTimeReqPayload::len(),
            Self::DeviceTimeAns(_) => DeviceTimeAnsPayload::len(),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        match *self {
            Self::LinkCheckReq(_) => &[],
            Self::LinkCheckAns(ref v) => v.build(),
            Self::LinkADRReq(ref v) => v.build(),
            Self::LinkADRAns(ref v) => v.build(),
            Self::DutyCycleReq(ref v) => v.build(),
            Self::DutyCycleAns(_) => &[],
            Self::RXParamSetupReq(ref v) => v.build(),
            Self::RXParamSetupAns(ref v) => v.build(),
            Self::DevStatusReq(_) => &[],
            Self::DevStatusAns(ref v) => v.build(),
            Self::NewChannelReq(ref v) => v.build(),
            Self::NewChannelAns(ref v) => v.build(),
            Self::RXTimingSetupReq(ref v) => v.build(),
            Self::RXTimingSetupAns(_) => &[],
            Self::TXParamSetupReq(ref v) => v.build(),
            Self::TXParamSetupAns(_) => &[],
            Self::DlChannelReq(ref v) => v.build(),
            Self::DlChannelAns(_) => &[],
            Self::DeviceTimeReq(_) => &[],
            Self::DeviceTimeAns(ref v) => v.build(),
        }
    }
}
impl SerializableMacCommand for MacCommandCreator {
    fn payload_bytes(&self) -> &[u8] {
        self.bytes()
    }

    fn cid(&self) -> u8 {
        match *self {
            MacCommandCreator::LinkCheckReq(_) => LinkCheckReqPayload::cid(),
            MacCommandCreator::LinkCheckAns(_) => LinkCheckAnsPayload::cid(),
            MacCommandCreator::LinkADRReq(_) => LinkADRReqPayload::cid(),
            MacCommandCreator::LinkADRAns(_) => LinkADRAnsPayload::cid(),
            MacCommandCreator::DutyCycleReq(_) => DutyCycleReqPayload::cid(),
            MacCommandCreator::DutyCycleAns(_) => DutyCycleAnsPayload::cid(),
            MacCommandCreator::RXParamSetupReq(_) => RXParamSetupReqPayload::cid(),
            MacCommandCreator::RXParamSetupAns(_) => RXParamSetupAnsPayload::cid(),
            MacCommandCreator::DevStatusReq(_) => DevStatusReqPayload::cid(),
            MacCommandCreator::DevStatusAns(_) => DevStatusAnsPayload::cid(),
            MacCommandCreator::NewChannelReq(_) => NewChannelReqPayload::cid(),
            MacCommandCreator::NewChannelAns(_) => NewChannelAnsPayload::cid(),
            MacCommandCreator::RXTimingSetupReq(_) => RXTimingSetupReqPayload::cid(),
            MacCommandCreator::RXTimingSetupAns(_) => RXTimingSetupAnsPayload::cid(),
            MacCommandCreator::TXParamSetupReq(_) => TXParamSetupReqPayload::cid(),
            MacCommandCreator::TXParamSetupAns(_) => TXParamSetupAnsPayload::cid(),
            MacCommandCreator::DlChannelReq(_) => DlChannelReqPayload::cid(),
            MacCommandCreator::DlChannelAns(_) => DlChannelAnsPayload::cid(),
            MacCommandCreator::DeviceTimeReq(_) => DeviceTimeReqPayload::cid(),
            MacCommandCreator::DeviceTimeAns(_) => DeviceTimeAnsPayload::cid(),
        }
    }

    fn payload_len(&self) -> usize {
        self.len()
    }
}

/// LinkCheckReqCreator serves for creating LinkCheckReq MacCommand.
///
/// # Examples
///
/// ```
/// let mut creator = lorawan::maccommandcreator::LinkCheckReqCreator::new();
/// let res = creator.build();
/// ```

#[derive(Debug, PartialEq, Eq)]
pub struct LinkCheckReqCreator {}

impl_mac_cmd_creator_boilerplate!(LinkCheckReqCreator, 0x02);

/// LinkCheckAnsCreator serves for creating LinkCheckAns MacCommand.
///
/// # Examples
///
/// ```
/// let mut creator = lorawan::maccommandcreator::LinkCheckAnsCreator::new();
/// let res = creator.set_margin(253).set_gateway_count(254).build();
/// ```

#[derive(Debug, PartialEq, Eq)]
pub struct LinkCheckAnsCreator {
    data: [u8; 3],
}

impl_mac_cmd_creator_boilerplate!(LinkCheckAnsCreator, 0x02, 3);

impl LinkCheckAnsCreator {
    /// Sets the margin of the LinkCheckAns to the provided value.
    ///
    /// # Argument
    ///
    /// * margin - margin  in  dB. The value is relative to the demodulation floor. The value 255
    /// is reserved.
    pub fn set_margin(&mut self, margin: u8) -> &mut Self {
        self.data[1] = margin;

        self
    }

    /// Sets the gateway count of the LinkCheckAns to the provided value.
    ///
    /// # Argument
    ///
    /// * gateway_count - the number of gateways that received the LinkCheckReq.
    pub fn set_gateway_count(&mut self, gw_cnt: u8) -> &mut Self {
        self.data[2] = gw_cnt;

        self
    }
}

/// LinkADRReqCreator serves for creating LinkADRReq MacCommand.
///
/// # Examples
///
/// ```
/// let mut creator = lorawan::maccommandcreator::LinkADRReqCreator::new();
/// let channel_mask_bytes = [0xc7, 0x0b];
/// let res = creator
///     .set_data_rate(0x05)
///     .unwrap()
///     .set_tx_power(0x03)
///     .unwrap()
///     .set_channel_mask(channel_mask_bytes)
///     .set_redundancy(0x37)
///     .build();
/// ```

#[derive(Debug, PartialEq, Eq)]
pub struct LinkADRReqCreator {
    data: [u8; 5],
}

impl_mac_cmd_creator_boilerplate!(LinkADRReqCreator, 0x03, 5);

impl LinkADRReqCreator {
    /// Sets the data rate of the LinkADRReq to the provided value.
    ///
    /// # Argument
    ///
    /// * data_rate - data rate index of the ADR request. The value must be between 0 and 15.
    pub fn set_data_rate(&mut self, data_rate: u8) -> Result<&mut Self, &str> {
        if data_rate > 0x0f {
            return Err("data_rate out of range");
        }
        self.data[1] &= 0x0f;
        self.data[1] |= data_rate << 4;

        Ok(self)
    }

    /// Sets the tx power of the LinkADRReq to the provided value.
    ///
    /// # Argument
    ///
    /// * tx_power - TX power index. The value must be between 0 and 15.
    pub fn set_tx_power(&mut self, tx_power: u8) -> Result<&mut Self, &str> {
        if tx_power > 0x0f {
            return Err("tx_power out of range");
        }
        self.data[1] &= 0xf0;
        self.data[1] |= tx_power & 0x0f;

        Ok(self)
    }

    /// Sets the channel mask of the LinkADRReq to the provided value.
    ///
    /// # Argument
    ///
    /// * channel_mask - instance of maccommands::ChannelMask or anything that can be converted
    /// into it.
    pub fn set_channel_mask<T: Into<ChannelMask>>(&mut self, channel_mask: T) -> &mut Self {
        let converted = channel_mask.into();
        self.data[2] = converted.as_ref()[0];
        self.data[3] = converted.as_ref()[1];

        self
    }

    /// Sets the redundancy of the LinkADRReq to the provided value.
    ///
    /// # Argument
    ///
    /// * redundancy - instance of maccommands::Redundancy or anything that can be converted
    /// into it.
    pub fn set_redundancy<T: Into<Redundancy>>(&mut self, redundancy: T) -> &mut Self {
        let converted = redundancy.into();
        self.data[4] = converted.raw_value();

        self
    }
}

/// LinkADRAnsCreator serves for creating LinkADRAns MacCommand.
///
/// # Examples
///
/// ```
/// let mut creator = lorawan::maccommandcreator::LinkADRAnsCreator::new();
/// let res = creator
///     .set_channel_mask_ack(true)
///     .set_data_rate_ack(true)
///     .set_tx_power_ack(true)
///     .build();
/// ```

#[derive(Debug, PartialEq, Eq)]
pub struct LinkADRAnsCreator {
    data: [u8; 2],
}

impl_mac_cmd_creator_boilerplate!(LinkADRAnsCreator, 0x03, 2);

impl LinkADRAnsCreator {
    /// Sets the channel mask acknowledgement of the LinkADRAns to the provided value.
    ///
    /// # Argument
    ///
    /// * ack - true meaning that the channel mask was acceptable or false otherwise.
    pub fn set_channel_mask_ack(&mut self, ack: bool) -> &mut Self {
        self.data[1] &= 0xfe;
        self.data[1] |= ack as u8;

        self
    }

    /// Sets the data rate acknowledgement of the LinkADRAns to the provided value.
    ///
    /// # Argument
    ///
    /// * ack - true meaning that the data rate was acceptable or false otherwise.
    pub fn set_data_rate_ack(&mut self, ack: bool) -> &mut Self {
        self.data[1] &= 0xfd;
        self.data[1] |= (ack as u8) << 1;

        self
    }

    /// Sets the tx power acknowledgement of the LinkADRAns to the provided value.
    ///
    /// # Argument
    ///
    /// * ack - true meaning that the tx power was acceptable or false otherwise.
    pub fn set_tx_power_ack(&mut self, ack: bool) -> &mut Self {
        self.data[1] &= 0xfb;
        self.data[1] |= (ack as u8) << 2;

        self
    }
}

/// DutyCycleReqCreator serves for creating DutyCycleReq MacCommand.
///
/// # Examples
///
/// ```
/// let mut creator = lorawan::maccommandcreator::DutyCycleReqCreator::new();
/// let res = creator.set_max_duty_cycle(0x0f).unwrap().build();
/// ```

#[derive(Debug, PartialEq, Eq)]
pub struct DutyCycleReqCreator {
    data: [u8; 2],
}

impl_mac_cmd_creator_boilerplate!(DutyCycleReqCreator, 0x04, 2);

impl DutyCycleReqCreator {
    /// Sets the max duty cycle of the DutyCycleReq to the provided value.
    ///
    /// # Argument
    ///
    /// * max_duty_cycle - the value used to determine the aggregated duty cycle using the formula
    /// `1 / (2 ** max_duty_cycle)`.
    pub fn set_max_duty_cycle(&mut self, max_duty_cycle: u8) -> Result<&mut Self, &str> {
        self.data[1] = max_duty_cycle;

        Ok(self)
    }
}

/// DutyCycleAnsCreator serves for creating DutyCycleAns MacCommand.
///
/// # Examples
///
/// ```
/// let creator = lorawan::maccommandcreator::DutyCycleAnsCreator::new();
/// let res = creator.build();
/// ```

#[derive(Debug, PartialEq, Eq)]
pub struct DutyCycleAnsCreator {}

impl_mac_cmd_creator_boilerplate!(DutyCycleAnsCreator, 0x04);

/// RXParamSetupReqCreator serves for creating RXParamSetupReq MacCommand.
///
/// # Examples
///
/// ```
/// let mut creator = lorawan::maccommandcreator::RXParamSetupReqCreator::new();
/// let res = creator
///     .set_dl_settings(0xcd)
///     .set_frequency(&[0x12, 0x34, 0x56])
///     .build();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct RXParamSetupReqCreator {
    data: [u8; 5],
}

impl_mac_cmd_creator_boilerplate!(RXParamSetupReqCreator, 0x05, 5);

impl RXParamSetupReqCreator {
    /// Sets the DLSettings of the RXParamSetupReq to the provided value.
    ///
    /// # Argument
    ///
    /// * dl_settings - instance of maccommands::DLSettings or anything that can be converted
    /// into it.
    pub fn set_dl_settings<T: Into<DLSettings>>(&mut self, dl_settings: T) -> &mut Self {
        let converted = dl_settings.into();
        self.data[1] = converted.raw_value();

        self
    }

    /// Sets the frequency of the RXParamSetupReq to the provided value.
    ///
    /// # Argument
    ///
    /// * frequency - instance of maccommands::Frequency or anything that can be converted
    /// into it.
    pub fn set_frequency<'a, T: Into<Frequency<'a>>>(&mut self, frequency: T) -> &mut Self {
        let converted = frequency.into();
        self.data[2..5].copy_from_slice(converted.as_ref());

        self
    }
}

/// RXParamSetupAnsCreator serves for creating RXParamSetupAns MacCommand.
///
/// # Examples
///
/// ```
/// let mut creator = lorawan::maccommandcreator::RXParamSetupAnsCreator::new();
/// let res = creator
///     .set_channel_ack(true)
///     .set_rx2_data_rate_ack(true)
///     .set_rx1_data_rate_offset_ack(true)
///     .build();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct RXParamSetupAnsCreator {
    data: [u8; 2],
}

impl_mac_cmd_creator_boilerplate!(RXParamSetupAnsCreator, 0x05, 2);

impl RXParamSetupAnsCreator {
    /// Sets the channel acknowledgement of the RXParamSetupAns to the provided value.
    ///
    /// # Argument
    ///
    /// * ack - true meaning that the channel was acceptable or false otherwise.
    pub fn set_channel_ack(&mut self, ack: bool) -> &mut Self {
        self.data[1] &= 0xfe;
        self.data[1] |= ack as u8;

        self
    }

    /// Sets the rx2 data rate acknowledgement of the RXParamSetupAns to the provided value.
    ///
    /// # Argument
    ///
    /// * ack - true meaning that the rx2 data rate was acceptable or false otherwise.
    pub fn set_rx2_data_rate_ack(&mut self, ack: bool) -> &mut Self {
        self.data[1] &= 0xfd;
        self.data[1] |= (ack as u8) << 1;

        self
    }

    /// Sets the rx1 data rate offset acknowledgement of the RXParamSetupAns to the provided value.
    ///
    /// # Argument
    ///
    /// * ack - true meaning that the rx1 data rate offset was acceptable or false otherwise.
    pub fn set_rx1_data_rate_offset_ack(&mut self, ack: bool) -> &mut Self {
        self.data[1] &= 0xfb;
        self.data[1] |= (ack as u8) << 2;

        self
    }
}

/// DevStatusReqCreator serves for creating DevStatusReq MacCommand.
///
/// # Examples
///
/// ```
/// let creator = lorawan::maccommandcreator::DevStatusReqCreator::new();
/// let res = creator.build();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct DevStatusReqCreator {}

impl_mac_cmd_creator_boilerplate!(DevStatusReqCreator, 0x06);

/// DevStatusAnsCreator serves for creating DevStatusAns MacCommand.
///
/// # Examples
///
/// ```
/// let mut creator = lorawan::maccommandcreator::DevStatusAnsCreator::new();
/// let res = creator.set_battery(0xfe).set_margin(-32).unwrap().build();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct DevStatusAnsCreator {
    data: [u8; 3],
}

impl_mac_cmd_creator_boilerplate!(DevStatusAnsCreator, 0x06, 3);

impl DevStatusAnsCreator {
    /// Sets the battery of the DevStatusAns to the provided value.
    ///
    /// # Argument
    ///
    /// * battery - the value to be used as the battery level. 0 means external enery source,
    /// 1 and 254 are the smallest and biggest values of normal battery reading, while 255
    /// indicates that the device failed to measure its battery level.
    pub fn set_battery(&mut self, battery: u8) -> &mut Self {
        self.data[1] = battery;

        self
    }

    /// Sets the margin of the DevStatusAns to the provided value.
    ///
    /// # Argument
    ///
    /// * margin - the value to be used as margin.
    pub fn set_margin(&mut self, margin: i8) -> Result<&mut Self, &str> {
        if !(-32..=31).contains(&margin) {
            return Err("margin out of range");
        }
        self.data[2] = ((margin << 2) as u8) >> 2;

        Ok(self)
    }
}

/// NewChannelReqCreator serves for creating NewChannelReq MacCommand.
///
/// # Examples
///
/// ```
/// let mut creator = lorawan::maccommandcreator::NewChannelReqCreator::new();
/// let res = creator
///     .set_channel_index(0x0f)
///     .set_frequency(&[0x12, 0x34, 0x56])
///     .set_data_rate_range(0x53)
///     .build();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct NewChannelReqCreator {
    data: [u8; 6],
}

impl_mac_cmd_creator_boilerplate!(NewChannelReqCreator, 0x07, 6);

impl NewChannelReqCreator {
    /// Sets the channel index of the NewChannelReq to the provided value.
    ///
    /// # Argument
    ///
    /// * channel_index - the value to be used as channel_index.
    pub fn set_channel_index(&mut self, channel_index: u8) -> &mut Self {
        self.data[1] = channel_index;

        self
    }

    /// Sets the frequency of the NewChannelReq to the provided value.
    ///
    /// # Argument
    ///
    /// * frequency - instance of maccommands::Frequency or anything that can be converted
    /// into it.
    pub fn set_frequency<'a, T: Into<Frequency<'a>>>(&mut self, frequency: T) -> &mut Self {
        let converted = frequency.into();
        self.data[2..5].copy_from_slice(converted.as_ref());

        self
    }

    /// Sets the data rate range of the NewChannelReq to the provided value.
    ///
    /// # Argument
    ///
    /// * data_rate_range - instance of maccommands::DataRateRange or anything that can be converted
    /// into it.
    pub fn set_data_rate_range<T: Into<DataRateRange>>(&mut self, data_rate_range: T) -> &mut Self {
        self.data[5] = data_rate_range.into().raw_value();

        self
    }
}

/// NewChannelAnsCreator serves for creating NewChannelAns MacCommand.
///
/// # Examples
///
/// ```
/// let mut creator = lorawan::maccommandcreator::NewChannelAnsCreator::new();
/// let res = creator
///     .set_channel_frequency_ack(true)
///     .set_data_rate_range_ack(true)
///     .build();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct NewChannelAnsCreator {
    data: [u8; 2],
}

impl_mac_cmd_creator_boilerplate!(NewChannelAnsCreator, 0x07, 2);

impl NewChannelAnsCreator {
    /// Sets the channel frequency acknowledgement of the NewChannelAns to the provided value.
    ///
    /// # Argument
    ///
    /// * ack - true meaning that the channel frequency was acceptable or false otherwise.
    pub fn set_channel_frequency_ack(&mut self, ack: bool) -> &mut Self {
        self.data[1] &= 0xfe;
        self.data[1] |= ack as u8;

        self
    }

    /// Sets the data rate range acknowledgement of the NewChannelAns to the provided value.
    ///
    /// # Argument
    ///
    /// * ack - true meaning that the data rate range was acceptable or false otherwise.
    pub fn set_data_rate_range_ack(&mut self, ack: bool) -> &mut Self {
        self.data[1] &= 0xfd;
        self.data[1] |= (ack as u8) << 1;

        self
    }
}

/// RXTimingSetupReqCreator serves for creating RXTimingSetupReq MacCommand.
///
/// # Examples
///
/// ```
/// let mut creator = lorawan::maccommandcreator::RXTimingSetupReqCreator::new();
/// let res = creator.set_delay(0x0f).unwrap().build();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct RXTimingSetupReqCreator {
    data: [u8; 2],
}

impl_mac_cmd_creator_boilerplate!(RXTimingSetupReqCreator, 0x08, 2);

impl RXTimingSetupReqCreator {
    /// Sets the delay of the RXTimingSetupReq to the provided value.
    ///
    /// # Argument
    ///
    /// * delay - the value to be used as delay.
    pub fn set_delay(&mut self, delay: u8) -> Result<&mut Self, &str> {
        if delay > 0x0f {
            return Err("delay out of range");
        }
        self.data[1] &= 0xf0;
        self.data[1] |= delay;

        Ok(self)
    }
}

/// RXTimingSetupAnsCreator serves for creating RXTimingSetupAns MacCommand.
///
/// # Examples
///
/// ```
/// let creator = lorawan::maccommandcreator::RXTimingSetupAnsCreator::new();
/// let res = creator.build();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct RXTimingSetupAnsCreator {}
impl_mac_cmd_creator_boilerplate!(RXTimingSetupAnsCreator, 0x08);

#[derive(Debug, PartialEq, Eq)]
pub struct TXParamSetupReqCreator {
    data: [u8; 1],
}
impl_mac_cmd_creator_boilerplate!(TXParamSetupReqCreator, 0x09, 1);

#[derive(Debug, PartialEq, Eq)]
pub struct TXParamSetupAnsCreator;
impl_mac_cmd_creator_boilerplate!(TXParamSetupAnsCreator, 0x09);

#[derive(Debug, PartialEq, Eq)]
pub struct DlChannelReqCreator {
    data: [u8; 4],
}
impl_mac_cmd_creator_boilerplate!(DlChannelReqCreator, 0x0A, 4);
#[derive(Debug, PartialEq, Eq)]
pub struct DlChannelAnsCreator;
impl_mac_cmd_creator_boilerplate!(DlChannelAnsCreator, 0x0A);
#[derive(Debug, PartialEq, Eq)]
pub struct DeviceTimeReqCreator;
impl_mac_cmd_creator_boilerplate!(DeviceTimeReqCreator, 0x0D);
#[derive(Debug, PartialEq, Eq)]
pub struct DeviceTimeAnsCreator {
    data: [u8; 5],
}
impl_mac_cmd_creator_boilerplate!(DeviceTimeAnsCreator, 0x0D, 5);
