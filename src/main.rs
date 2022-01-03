extern crate clap;
use anyhow::Context;
use ba_postcard_proto as proto;
use chrono::Timelike;
use clap::{App, Arg, SubCommand};

use std::time::Duration;

struct Device {
    port: Box<dyn serialport::SerialPort>,
}

impl Device {
    pub fn new(port: Box<dyn serialport::SerialPort>) -> Self {
        Device { port: port }
    }

    pub fn command_response(
        &mut self,
        message: &proto::Command,
    ) -> anyhow::Result<proto::Response> {
        let mut buf = [0; 64];

        let serialize = postcard::to_slice_cobs(message, &mut buf)?;
        self.port.write_all(serialize)?;
        self.port.flush()?;
        let _count = self.port.read(&mut buf)?;

        postcard::from_bytes_cobs::<proto::Response>(&mut buf).context("deserialization error")
    }
}

fn main() {
    let matches = App::new("blueacro communication CLI")
        .version("1.0")
        .author("Yann Ramin <yann@theatr.us>")
        .about("Communicate over USB or Serial ports to control modules")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("Sets a communication port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            SubCommand::with_name("get_time")
                .about("interrogates the current time from the device"),
        )
        .subcommand(
            SubCommand::with_name("set_time")
                .about("Sets the time on the target device to the local time of this system"),
        )
        .get_matches();

    let port = matches.value_of("port").unwrap_or("/dev/ttyACM1");
    let port = serialport::new(port, 9600)
        .timeout(Duration::from_secs(1))
        .open()
        .expect(format!("failed to open port {}", port).as_str());

    let mut device = Device::new(port);

    if let Some(_matches) = matches.subcommand_matches("get_time") {
        let message = proto::Command::QueryTime;
        let incoming = device.command_response(&message);
        println!("{:?}", incoming);
    } else if let Some(_) = matches.subcommand_matches("set_time") {
        let now = chrono::Local::now();
        let message = proto::Command::SetTime(proto::SetTime {
            minutes: now.time().minute()  as u8,
            hours: now.time().hour() as u8,
            seconds: now.time().second() as u8,
        });
        

        println!("{:?}", device.command_response(&message));
    }
}
