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

    signals: Vec<Signal>,
}

pub struct Signal {
    signal_name: String,
    signal_type: String,
    signal_offset: u16,
    signal_status: bool,
}

impl Signal {
    pub fn as_text(&self) -> String {
        match &self.signal_status {
            true => "ON".to_string(),
            false => "OFF".to_string(),
        }
    }

    pub fn new(signal_name: String, signal_type: String, signal_offset: u16) -> Signal {
        println!(
            "Creating signal: {}, type: {}, offset: {}",
            signal_name, signal_type, signal_offset
        );

        Signal {
            signal_name,
            signal_type,
            signal_offset,
            signal_status: false,
        }
    }

    pub fn get_signal_name(&self) -> &String {
        &self.signal_name
    }

    pub fn get_signal_offset(&self) -> &u16 {
        &self.signal_offset
    }
    pub fn get_signal_status(&self) -> &bool {
        //TODO: Make this updated via a scanner thread instead of via direct calls through
        //SignalDevice
        &self.signal_status
    }
}

impl SignalDevice {
    // Search through the hash of all signals, find the one by name referenced, and return
    // a reference to the struct <Signal>
    pub fn get_signal(&mut self) -> &Signal {
        //TODO: Make this return a named signal or ERR
        &self.signals.first().unwrap()
    }

    pub fn get_signal_directly(&mut self) -> bool {
        //TODO Make this use a named signal or return ERR
        let signal: &Signal = self.get_signal();
        let signal_offset: u16 = signal.get_signal_offset().clone();
        get_signal_status(&mut self.client, signal_offset)
    }

    pub fn get_device(&mut self) -> &String {
        &self.device
    }

    pub fn get_resource_location(&mut self) -> &String {
        &self.resource_location
    }
}

pub fn new(device_name: String) -> SignalDevice {
    let resource_location: String = format!("./thingy/resources/{}.yaml", device_name);
    println!("Creating device: {}", device_name);
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

    // Get signals information out of the config file and create signal objects
    // for each record in the signals: hash
    let mut signals: Vec<Signal> = Vec::new();

    for signal_vec in device_conf["signals"].clone().as_vec() {
        for signal in signal_vec {
            let signal_name: String = signal["name"]
                .clone()
                .into_string()
                .unwrap_or(format!("invalid_signal_name"));

            let signal_type: String = signal["type"]
                .clone()
                .into_string()
                .unwrap_or(format!("invalid_signal_type"));

            let signal_offset: u16 = signal["offset"].clone().as_i64().unwrap_or(0) as u16;

            let sig: Signal = Signal::new(signal_name, signal_type, signal_offset);
            signals.push(sig);
        }
    }

    let client = tcp::Transport::new(&coupler).unwrap();

    SignalDevice {
        device: coupler.to_string(),
        client: client,
        resource_location: resource_location,
        signals: signals,
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
