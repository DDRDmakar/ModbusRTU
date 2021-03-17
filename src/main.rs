//------------------------------------------------------------------------------
// author:	Nikita Makarevich
// email:	nikita.makarevich@spbpu.com
// 2021
//------------------------------------------------------------------------------
// Simple Modbus RTU server
//------------------------------------------------------------------------------

use structopt::StructOpt;
use serialport;
use std::path::PathBuf;
use std::time::Duration;
use std::process;
//use std::fs::File;


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
	//println!("{:#?}", opt);
	
	let ports = serialport::available_ports().expect("В системе не обнаружено последовательных портов");

	let port_name = match ports.iter().find(|p| p.port_name == opt.port) {
		Some(p) => p.port_name.as_str(),
		None    => {
			eprintln!("Последовательный порт \"{}\" не существует.", opt.port);
			eprintln!("Список существующих:");
			for (i, p) in ports.iter().enumerate() {
				eprintln!("\t{}: {}", i, p.port_name);
			}
			process::exit(1);
		}
	};
	
	let port = serialport::new(port_name, opt.baudrate)
		.timeout(Duration::from_millis(10))
		.open().expect("Не удалось открыть порт");

	
	
	Ok(())
}
