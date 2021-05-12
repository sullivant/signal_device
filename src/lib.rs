extern crate modbus;
extern crate yaml_rust;

use log::{info, warn};
use modbus::tcp;
use modbus::Client;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::time;
use yaml_rust::YamlLoader;

pub struct SignalDevice {
    device_name: String,     // The provided name of this device
    coupler_address: String, // The IP/Hostname of the device
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
        info!(
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

    pub fn get_signal_type(&self) -> &String {
        &self.signal_type
    }

    pub fn set_signal_status(&mut self, new_val: bool) {
        self.signal_status = new_val;
    }

    pub fn get_signal_status(&self) -> &bool {
        &self.signal_status
    }
}

impl std::fmt::Debug for SignalDevice {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SignalDevice")
            .field("device_name", &self.device_name)
            .field("coupler_address", &self.coupler_address)
            .finish()
    }
}

impl fmt::Display for SignalDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "For signal device {} using address of: {}",
            &self.device_name, &self.coupler_address
        )
    }
}

impl SignalDevice {
    // Search through the hash of all signals, find the one by name referenced, and return
    // a reference to the struct <Signal> or an Err with the details about the missing
    // signal name
    pub fn get_signal(&mut self, signal_name: &str) -> Result<&Signal, String> {
        for s in &self.signals {
            if s.signal_name == *signal_name {
                return Ok(&s);
            }
        }

        Err(format!("Unable to find signal: '{}'", signal_name))
    }

    pub fn get_signal_mut(&mut self, signal_name: &str) -> Result<&mut Signal, String> {
        for s in &mut self.signals {
            if s.signal_name == *signal_name {
                return Ok(s);
            }
        }

        Err(format!("Unable to find signal: '{}'", signal_name))
    }

    // Returns a Result<> containing either the value of the signal when the coupler
    // is asked directly  (via get_signal_status()) or an Err containing the details
    pub fn get_signal_directly(&mut self, signal_name: &str) -> Result<bool, String> {
        let signal: &Signal = self.get_signal(signal_name)?;
        let signal_offset: u16 = *signal.get_signal_offset();
        Ok(get_signal_status(&mut self.client, signal_offset))
    }

    pub fn get_name(&mut self) -> &String {
        &self.device_name
    }

    pub fn get_coupler_address(&mut self) -> &String {
        &self.coupler_address
    }
    pub fn set_coupler_address(&mut self, address: String) {
        self.coupler_address = address;
    }

    pub fn get_resource_location(&mut self) -> &String {
        &self.resource_location
    }

    pub fn refresh_signals(&mut self) {
        for signal in self.signals.iter_mut() {
            let signal_offset: u16 = *signal.get_signal_offset();
            let status = get_signal_status(&mut self.client, signal_offset);
            signal.set_signal_status(status);
        }
    }

    pub fn new(device_name: &str) -> Result<SignalDevice, String> {
        // The tcp_config object will let us specify a timeout
        //let mut tcp_config = tcp::Config::default();
        //tcp_config.tcp_connect_timeout = Some(time::Duration::from_millis(1000));
        let tcp_config = modbus::Config {
            tcp_connect_timeout: Some(time::Duration::from_millis(1000)),
            ..Default::default()
        };

        let resource_location: String = format!("./thingy/resources/{}.yaml", device_name);
        info!("Creating signal device: {}", device_name);
        info!("Using device configuration at: {}", resource_location);
        let file = match File::open(resource_location.clone()) {
            Ok(f) => f,
            Err(e) => return Err(format!("Unable to open device configuration: {}", e)),
        };

        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader
            .read_to_string(&mut contents)
            .expect("Unable to read input file.");

        let device_yaml = match YamlLoader::load_from_str(&contents) {
            Ok(c) => c,
            Err(e) => return Err(format!("Unable parse device configuration: {}", e)),
        };
        let device_conf = &device_yaml[0];

        // Get device coupler information
        let coupler_raw = &device_conf["device"]["coupler"];
        let coupler = match coupler_raw.as_str() {
            Some(s) => s,
            _ => {
                warn!("Using default coupler IP of 127.0.0.1");
                "127.0.0.1"
            }
        };

        // Get signals information out of the config file and create signal objects
        // for each record in the signals: hash
        let mut signals: Vec<Signal> = Vec::new();

        info!("Connecting to coupler at {}", coupler);
        let client = match tcp::Transport::new_with_cfg(&coupler, tcp_config) {
            Ok(c) => c,
            Err(e) => return Err(format!("Unable to create tcp connection: {}", e)),
        };

        if let Some(signal_vec) = device_conf["signals"].clone().as_vec() {
            for signal in signal_vec {
                let signal_name: String = signal["name"]
                    .clone()
                    .into_string()
                    .unwrap_or_else(|| "invalid_signal_name".to_string());

                let signal_type: String = signal["type"]
                    .clone()
                    .into_string()
                    .unwrap_or_else(|| "invalid_signal_type".to_string());

                let signal_offset: u16 = signal["offset"].clone().as_i64().unwrap_or(0) as u16;

                let sig: Signal = Signal::new(signal_name, signal_type, signal_offset);
                signals.push(sig);
            }
        }

        Ok(SignalDevice {
            device_name: device_name.to_string(),
            coupler_address: coupler.to_string(),
            client,
            resource_location,
            signals,
        })
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
