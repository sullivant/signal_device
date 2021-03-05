extern crate modbus;
extern crate yaml_rust;

use modbus::tcp;
use modbus::Client;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use yaml_rust::YamlLoader;

pub struct SignalDevice {
    device: String, // The IP/Hostname of the device
    client: tcp::Transport,
    resource_location: String, // the location of the configuration file for this device

    // TODO: Turn these into an array of signals structs
    signal_num: u16,
    signal_status: bool,
}

pub struct Signal {
    signal_num: u16,
    signal_name: String,
    signal_status: bool,
}

impl Signal {
    pub fn as_text(&self) -> String {
        match &self.signal_status {
            true => "ON".to_string(),
            false => "OFF".to_string(),
        }
    }
}

impl SignalDevice {
    pub fn get_signal(&mut self) -> bool {
        //let mut client = tcp::Transport::new(&self.device).unwrap();
        let val: bool = get_signal_status(&mut self.client, self.signal_num);

        val
    }

    pub fn get_device(&mut self) -> &String {
        &self.device
    }
}

pub fn new(device_name: String) -> SignalDevice {
    let resource_location: String = format!("./resources/{}.yaml", device_name);
    println!("Using device configuration at: {}", resource_location);
    let file = File::open(resource_location.clone()).expect("Unable to open file.");
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader
        .read_to_string(&mut contents)
        .expect("Unable to read input file.");

    let device_yaml = YamlLoader::load_from_str(&contents).unwrap();
    let device_conf = &device_yaml[0];

    // Get device coupler information
    let coupler_raw = &device_conf["device"]["coupler"];
    let coupler = coupler_raw.as_str().unwrap_or("127.0.0.1");

    // Get signals information
    let signal_offset_raw = &device_conf["signals"]["diInputSensor"]["offset"];
    let signal_offset: u16 = signal_offset_raw.as_i64().unwrap_or(0) as u16;

    println!(
        "Creating device at: {} with signal offset of: {}",
        coupler, signal_offset
    );

    let client = tcp::Transport::new(&coupler).unwrap();
    SignalDevice {
        device: coupler.to_string(),
        client: client,
        resource_location: resource_location,
        signal_num: signal_offset,
        signal_status: false,
    }
}

// Reads one discrete input located at addr
//
// Parameters:
//  &mut client: mutable reference to a modbus::Transport client connection
//  addr: a u16 address location, zero indexed
//
// Returns:
//  true: if coil is on
//  false: if coil is off
pub fn get_signal_status(client: &mut modbus::Transport, addr: u16) -> bool {
    let mut retval: bool = false;

    for (_n, i) in client
        .read_discrete_inputs(addr, 1)
        .expect("IO Error")
        .iter()
        .enumerate()
    {
        match i {
            modbus::Coil::On => retval = true,
            modbus::Coil::Off => retval = false,
        }
    }

    retval
}
