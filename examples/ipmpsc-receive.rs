#![deny(warnings)]

use clap::{App, Arg};
use ipmpsc::{Receiver, SharedRingBuffer, ShmDeserializer, ShmZeroCopyDeserializer};
use serde::{Deserialize};

#[derive(Debug)]
pub struct BincodeZeroCopyDeserializer<T>(pub T);

impl<'de, T> ShmZeroCopyDeserializer<'de> for BincodeZeroCopyDeserializer<T>
where
    T: Deserialize<'de>,
{
    fn deserialize_from_bytes(bytes: &'de [u8]) -> ipmpsc::Result<Self> {
        Ok(Self(bincode::deserialize::<T>(bytes)?))
    }
}

#[derive(Debug)]
pub struct BincodeDeserializer<T>(pub T);

impl<T> ShmDeserializer for BincodeDeserializer<T>
where T: for<'de> Deserialize<'de>
{
    fn deserialize_from_bytes<'de>(bytes: &'de [u8]) -> ipmpsc::Result<Self> {
        Ok(Self(bincode::deserialize::<T>(bytes)?))
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("ipmpsc-send")
        .about("ipmpsc sender example")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::with_name("map file")
                .help(
                    "File to use for shared memory ring buffer.  \
                     This file will be cleared if it already exists or created if it doesn't.",
                )
                .required(true),
        )
        .arg(
            Arg::with_name("zero copy")
                .long("zero-copy")
                .help("Use zero-copy deserialization"),
        )
        .get_matches();

    let map_file = matches.value_of("map file").unwrap();
    let mut rx = Receiver::new(SharedRingBuffer::create(map_file, 32 * 1024)?);
    let zero_copy = matches.is_present("zero copy");

    println!(
        "Ready!  Now run `cargo run --example ipmpsc-send {}` in another terminal.",
        map_file
    );

    loop {
        if zero_copy {
            println!("received {:?}", rx.zero_copy_context().recv::<BincodeZeroCopyDeserializer<&str>>()?);
        } else {
            println!("received {:?}", rx.recv::<BincodeDeserializer<String>>()?);
        }
    }
}
