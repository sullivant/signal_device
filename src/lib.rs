#[macro_use]
extern crate clap;
extern crate modbus;

pub fn signals_hi() {
    println!("Hello from signals");
}

use clap::App;
use modbus::tcp;
use modbus::Client;

#[derive(Debug, Copy, Clone)]
struct Signal {
    signal_num: usize,
    signal_status: bool,
}
impl Signal {
    fn as_text(&self) -> String {
        match &self.signal_status {
            true => "ON".to_string(),
            false => "OFF".to_string(),
        }
    }
}

// Reads one coil located at addr
//
// Parameters:
//  &mut client: mutable reference to a modbus::Transport client connection
//  addr: a u16 address location, zero indexed
//
// Returns:
//  true: if coil is on
//  false: if coil is off
pub fn get_modbus_data(client: &mut modbus::Transport, addr: u16) -> bool {
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

pub fn get_signal() {
    let matches = App::new("modbus_client")
        .author("Thomas Sullivan")
        .version(&crate_version!()[..])
        .about("Modbus TCP Scanner")
        .args_from_usage("<DEVICE> 'The IP address or hostname of the device.")
        .get_matches();

    let device = matches.value_of("DEVICE").unwrap_or("192.168.0.1");

    println!("Using device address of: {}", device);

    let mut client = tcp::Transport::new(device).unwrap();

    let addr: u16 = 16;

    let val: bool = get_modbus_data(&mut client, addr);

    println!("{:?}", val);
}
