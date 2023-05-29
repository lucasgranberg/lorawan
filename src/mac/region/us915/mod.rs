pub struct Us915;

impl crate::mac::Region for Us915 {
    fn default_channels() -> u8 {
        72
    }

    fn mandatory_frequencies() -> &'static [u32] {
        todo!()
    }

    fn min_data_rate_join_req() -> crate::mac::types::DR {
        todo!()
    }

    fn max_data_rate_join_req() -> crate::mac::types::DR {
        todo!()
    }

    fn min_data_rate() -> crate::mac::types::DR {
        todo!()
    }

    fn max_data_rate() -> crate::mac::types::DR {
        todo!()
    }

    fn default_data_rate() -> crate::mac::types::DR {
        todo!()
    }

    fn default_coding_rate() -> crate::device::radio::types::CodingRate {
        todo!()
    }

    fn default_rx2_frequency() -> u32 {
        todo!()
    }

    fn default_rx2_data_rate() -> crate::mac::types::DR {
        todo!()
    }

    fn max_eirp() -> u8 {
        todo!()
    }

    fn min_frequency() -> u32 {
        todo!()
    }

    fn max_frequency() -> u32 {
        todo!()
    }

    fn convert_data_rate(
        dr: crate::mac::types::DR,
    ) -> Result<crate::device::radio::types::Datarate, super::Error> {
        todo!()
    }

    fn get_receive_window(
        rx_dr_offset: crate::mac::types::DR,
        downstream_dr: crate::mac::types::DR,
    ) -> crate::mac::types::DR {
        todo!()
    }

    fn supports_tx_param_setup() -> bool {
        todo!()
    }

    fn modify_dbm(
        tx_power: u8,
        cur_dbm: Option<u8>,
        max_eirp: u8,
    ) -> Result<Option<u8>, super::Error> {
        todo!()
    }

    fn default_rx1_data_rate_offset() -> u8 {
        todo!()
    }
}
