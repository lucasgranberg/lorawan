// Copyright (c) 2018-2020 Ivaylo Petrov
//
// Licensed under the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//
// Author: Ivaylo Petrov <ivajloip@gmail.com>

use super::maccommands::*;
use SerializableMacCommand;

macro_rules! impl_mac_cmd_creator_boilerplate {
    ($type:ident, $cid:expr) => {
        impl Default for $type {
            fn default() -> Self {
                Self {}
            }
        }

        impl $type {
            /// Creates a new instance of the class.
            pub fn new() -> Self {
                Default::default()
            }

            /// Returns the serialized version of the class as bytes.
            pub fn build(&self) -> &[u8] {
                &[$cid]
            }
        }

        impl_mac_cmd_payload!($type);
    };

    ($type:ident, $cid:expr, $len:expr) => {
        impl Default for $type {
            fn default() -> Self {
                let data = [0; $len];
                Self { data }
            }
        }

        impl $type {
            /// Creates a new instance of the class.
            pub fn new() -> Self {
                let mut data = [0; $len];
                data[0] = $cid;
                Self { data }
            }

            /// Returns the serialized version of the class as bytes.
            pub fn build(&self) -> &[u8] {
                &self.data[..]
            }

            pub fn get_data(self) -> [u8; $len] {
                self.data
            }
        }

        impl_mac_cmd_payload!($type);
    };
}

macro_rules! impl_mac_cmd_payload {
    ($type:ident) => {
        impl crate::encoding::maccommands::SerializableMacCommand for $type {
            /// Bytes of the SerializableMacCommand without the cid.
            fn payload_bytes(&self) -> &[u8] {
                &self.build()[1..]
            }

            /// The cid of the SerializableMacCommand.
            fn cid(&self) -> u8 {
                self.build()[0]
            }

            /// Length of the SerializableMacCommand without the cid.
            fn payload_len(&self) -> usize {
                self.build().len() - 1
            }
        }
    };
}
pub(crate) use impl_mac_cmd_creator_boilerplate;
pub(crate) use impl_mac_cmd_payload;

pub fn build_mac_commands<'a, 'b, 'c, T: AsMut<[u8]>>(
    cmds: &'a [&'b dyn SerializableMacCommand],
    mut out: T,
) -> Result<usize, &'c str> {
    let res = out.as_mut();
    if mac_commands_len(cmds) > res.len() {
        return Err("failed to serialize mac commands in provided buffer: too small");
    }
    let mut i = 0;
    for mc in cmds {
        res[i] = mc.cid();
        let l = mc.payload_len();
        res[i + 1..i + l + 1].copy_from_slice(mc.payload_bytes());
        i += l + 1;
    }
    Ok(i)
}
