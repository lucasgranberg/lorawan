# LoRaWAN Revision 1.0.4 implemented in Rust

[![CI](https://github.com/lucasgranberg/lorawan/actions/workflows/ci.yaml/badge.svg)](https://github.com/lucasgranberg/lorawan/actions/workflows/ci.yaml)

Provide end device support for LoRaWAN revision 1.0.4 in Rust, as specified in the following documents:

- <a href="https://resources.lora-alliance.org/technical-specifications/ts001-1-0-4-lorawan-l2-1-0-4-specification">Specification</a>
- <a href="https://resources.lora-alliance.org/technical-specifications/rp002-1-0-4-regional-parameters">Regional Parameters</a>

Currently supported:
- Class A; future support for Class B and C planned;
- Dynamic and fixed channel plans;
- EU868 and US915 regions; future support for additional regions planned;
- end device only, supporting communication with gateways and network applications.

The implementation maintains a clear distinction between the features it provides and those which must be provided by the caller.  This allows it to be used with a range of embedded frameworks and LoRa boards.  While the current <a href="https://github.com/lucasgranberg/lorawan-pilot">example application</a> uses features of the Embassy embedded framework, other embedded frameworks should work equally as well.

The following embedded framework functionality must be provided by the caller (see <a href="https://github.com/lucasgranberg/lorawan/blob/main/src/device/mod.rs">device aspects</a> for more detail):

- timer;
- LoRa radio;
- random number generator;
- non-volatile storage.

The external API used to establish a session between the end device and a network server (join operation) and subsequently transmit data to a network application (data operation) is detailed in the public functions of the Mac implementation <a href="https://github.com/lucasgranberg/lorawan/blob/main/src/mac/mod.rs">here</a>.

Important goals for the Class A feature:

- keep power consumption very low within end devices between what are often widely time-separated end device transmissions involving sensor readings;
- hide the complexities of join operations to gateways which only support a limited number of channels (an issue mainly associated with fixed channel plans).

### Chat

A public chat on LoRa/LoRaWAN topics using Rust is here:

- <a href="https://matrix.to/#/#public-lora-wan-rs:matrix.org">Matrix room</a>
