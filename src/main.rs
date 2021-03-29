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
	/// Serial port name
	#[structopt(short, long)]
	port: String,
	/// Baud rate
	#[structopt(short, long)]
	baudrate: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>>  {
	let opt = Opt::from_args();
	dbg!(&opt);
	
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

	println!("Выбрано имя порта: {}", port_name);
	
	let port = serialport::new(port_name, opt.baudrate)
		.timeout(Duration::from_millis(1000))
		.parity(serialport::Parity::None)
		.open().expect("Не удалось открыть порт");

	let mut server = server::Server::new(port);
	server.start()?;
	
	Ok(())
}
