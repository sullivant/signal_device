extern crate modbus;
extern crate yaml_rust;

use modbus::tcp;
use modbus::Client;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

pub struct SignalDevice {
    device: String, // The IP/Hostname of the device
    client: tcp::Transport,

    // TODO: these will be an array built by config file
    // to also contain name
    signal_num: u16,
    signal_status: bool,
}
impl SignalDevice {
    pub fn as_text(&self) -> String {
        match &self.signal_status {
            true => "ON".to_string(),
            false => "OFF".to_string(),
        }
    }

    pub fn get_signal(&mut self) -> bool {
        //let mut client = tcp::Transport::new(&self.device).unwrap();
        let val: bool = get_signal_status(&mut self.client, self.signal_num);

        val
    }

    pub fn get_device(&mut self) -> &String {
        &self.device
    }
}

pub fn new(device: String) -> SignalDevice {
    let s = "
signals:
  diInputSensor:
    name: diInputSensor
    type: input
    offset: 16
";

    let device_yaml = YamlLoader::load_from_str(s).unwrap();
    let device_conf = &device_yaml[0];

    let signal_offset_raw = &device_conf["signals"]["diInputSensor"]["offset"];
    let signal_offset: u16 = signal_offset_raw.as_i64().unwrap_or(0) as u16;

    println!(
        "Creating device at: {} with signal offset of: {}",
        device, signal_offset
    );

    let client = tcp::Transport::new(&device).unwrap();
    SignalDevice {
        device,
        client: client,
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
