// Copyright (c) 2018,2020 Ivaylo Petrov
//
// Licensed under the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//
// author: Ivaylo Petrov <ivajloip@gmail.com>

pub trait SerializableMacCommand {
    fn payload_bytes(&self) -> &[u8];
    fn cid(&self) -> u8;
    fn payload_len(&self) -> usize;
}

/// Calculates the len in bytes of a sequence of mac commands, including th CIDs.
pub fn mac_commands_len(cmds: &[&dyn SerializableMacCommand]) -> usize {
    cmds.iter().map(|mc| mc.payload_len() + 1).sum()
}

macro_rules! mac_cmd_zero_len {
    (

        $(
            $(#[$outer:meta])*
            struct $type:ident[cmd=$name:ident, cid=$cid:expr, uplink=$uplink:expr]
            )*
    ) => {
        $(
            $(#[$outer])*
            pub struct $type();

            impl $type {
                pub fn new(_: &[u8]) -> Result<$type, &str> {
                    Ok($type())
                }

                pub fn new_as_mac_cmd<'a>(data: &[u8]) -> Result<(MacCommand<'a>, usize), &str> {
                    Ok((MacCommand::$name($type::new(data)?), 0))
                }

                pub const fn cid() -> u8 {
                    $cid
                }

                pub const fn uplink() -> bool {
                    $uplink
                }

                pub const fn len() -> usize {
                    0
                }
            }
        )*

        fn parse_zero_len_mac_cmd<'a, 'b>(data: &'a [u8], uplink: bool) -> Result<(usize, MacCommand<'a>), &'b str> {
            match (data[0], uplink) {
                $(
                    ($cid, $uplink) => Ok((0, MacCommand::$name($type::new(&[])?))),
                )*
                _ => Err("uknown mac command")
            }
        }
    }
}

macro_rules! mac_cmds {
    (

        $(
            $(#[$outer:meta])*
            struct $type:ident[cmd=$name:ident, cid=$cid:expr, uplink=$uplink:expr, size=$size:expr]
            )*
    ) => {
        $(
            $(#[$outer])*
            pub struct $type<'a>(&'a [u8]);

            impl<'a> $type<'a> {
                /// Creates a new instance of the mac command if there is enought data.
                pub fn new<'b>(data: &'a [u8]) -> Result<$type<'a>, &'b str> {
                    if data.len() < $size {
                        Err("incorrect size for")
                    } else {
                        Ok($type(&data))
                    }
                }

                pub fn new_as_mac_cmd<'b>(data: &'a [u8]) -> Result<(MacCommand<'a>, usize), &'b str> {
                    Ok((MacCommand::$name($type::new(data)?), $size))
                }

                /// Command identifier.
                pub const fn cid() -> u8 {
                    $cid
                }

                /// Sent by end device or sent by network server.
                pub const fn uplink() -> bool {
                    $uplink
                }

                /// length of the payload of the mac command.
                pub const fn len() -> usize {
                    $size
                }
            }
        )*

        fn parse_one_mac_cmd<'a, 'b>(data: &'a [u8], uplink: bool) -> Result<(usize, MacCommand<'a>), &'b str> {
            match (data[0], uplink) {
                $(
                    ($cid, $uplink) if data.len() > $size => Ok(($size, MacCommand::$name($type::new(&data[1.. 1 + $size])?))),
                )*
                _ => parse_zero_len_mac_cmd(data, uplink)
            }
        }
    }
}

macro_rules! create_ack_fn {
    (
        $(#[$outer:meta])*
        $fn_name:ident, $offset:expr
    ) => (
        $(#[$outer])*
        pub fn $fn_name(&self) -> bool {
            self.0[0] & (0x01 << $offset) != 0
        }
    )
}

macro_rules! create_value_reader_fn {
    (
        $(#[$outer:meta])*
        $fn_name:ident, $index:expr
    ) => (
        $(#[$outer])*
        pub fn $fn_name(&self) -> u8 {
            self.0[$index]
        }
    )
}
pub(crate) use create_ack_fn;
pub(crate) use create_value_reader_fn;
pub(crate) use mac_cmd_zero_len;
pub(crate) use mac_cmds;

/// Parses bytes to mac commands if possible.
///
/// Could return error if some values are out of range or the payload does not end at mac command
/// boundry.
/// # Argument
///
/// * bytes - the data from which the MAC commands are to be built.
/// * uplink - whether the packet is uplink or downlink.
///
/// # Examples
///
/// ```
/// let mut data = vec![0x02, 0x03, 0x00];
/// let mac_cmds: Vec<lorawan::maccommands::MacCommand> =
///     lorawan::maccommands::parse_mac_commands(&data[..], true).collect();
/// ```
pub fn parse_mac_commands(data: &[u8], uplink: bool) -> MacCommandIterator {
    MacCommandIterator {
        index: 0,
        data,
        uplink,
    }
}

/// Implementation of iterator for mac commands.
pub struct MacCommandIterator<'a> {
    pub(crate) data: &'a [u8],
    pub(crate) index: usize,
    pub(crate) uplink: bool,
}
