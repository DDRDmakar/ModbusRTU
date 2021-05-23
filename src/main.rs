//------------------------------------------------------------------------------
// author:	Nikita Makarevich (aka DDRDmakar)
// email:	makarevich.98@mail.ru
// 2021
// This code is under MIT license (see LICENSE.txt)
//------------------------------------------------------------------------------
// Простой сервер Modbus RTU
//------------------------------------------------------------------------------
use std::path::PathBuf;
use std::time::Duration;

use structopt::StructOpt;
use serialport::{ SerialPort, Parity };

mod server;

extern crate num;
#[macro_use]
extern crate num_derive;

#[derive(Debug, StructOpt)]
#[structopt(name = "Modbus RTU", about = "parameters")]
struct Opt {
	/// Input file
	#[structopt(parse(from_os_str), default_value="")]
	ifile: PathBuf,
	/// Slave id
	#[structopt(short, long, default_value="1")]
	slave_id: u8,
	/// Serial port name
	#[structopt(short, long)]
	port: String,
	/// Baud rate
	#[structopt(short, long, default_value="9600")]
	baudrate: u32,
	/// Serial port parity
	#[structopt(short="a", long, default_value="even")]
	parity: String,
	/// Timeout in ms
	#[structopt(short, long, default_value="1000")]
	timeout: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>>  {
	let opt = Opt::from_args();
	
	let ports = serialport::available_ports().expect("В системе не обнаружено последовательных портов");

	let port_name = match ports.iter().find(|p| p.port_name == opt.port) {
		Some(p) => p.port_name.as_str(),
		None    => {
			eprintln!("Внимание! Последовательный порт \"{}\" не найден.", opt.port);
			eprintln!("Список существующих:");
			if ports.len() > 0 {
				for (i, p) in ports.iter().enumerate() {
					eprintln!("\t{}: {}", i, p.port_name);
				}
			}
			else { eprintln!("[портов не найдено]"); }
			opt.port.as_str()
		},
	};
	let parity = match opt.parity.to_lowercase().as_str() {
		"even" => Parity::Even,
		"odd"  => Parity::Odd,
		"none" => Parity::None,
		&_     => panic!("Неверно указана чётность. Используйте значения: Even, Odd и None.")
	};

	let port = serialport::new(port_name, opt.baudrate)
		.timeout(Duration::from_millis(opt.timeout))
		.parity(parity)
		.open().expect("Не удалось открыть порт");

	display_port_settings(&port);

	let mut server = server::Server::new(port, opt.slave_id);
	server.start()?;
	
	Ok(())
}

fn display_port_settings(port: &Box<dyn SerialPort>) {
	println!("================[ Serial port ]==================");
	println!("name:         {:?}", port.name().unwrap());
	println!("baud rate:    {:?}", port.baud_rate().unwrap());
	println!("data bits:    {:?}", port.data_bits().unwrap());
	println!("parity:       {:?}", port.parity().unwrap());
	println!("stop bits:    {:?}", port.stop_bits().unwrap());
	println!("flow control: {:?}", port.flow_control().unwrap());
	println!("timeout:      {:?} ms", port.timeout().as_millis());
	println!("=================================================");
	
}
