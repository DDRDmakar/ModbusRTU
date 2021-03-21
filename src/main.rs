//------------------------------------------------------------------------------
// author:	Nikita Makarevich
// email:	nikita.makarevich@spbpu.com
// 2021
//------------------------------------------------------------------------------
// Simple Modbus RTU server
//------------------------------------------------------------------------------

use std::path::PathBuf;
use std::time::Duration;
use std::process;
use std::io::BufWriter;
use std::io::Write;

use byteorder::{ByteOrder, BigEndian, LittleEndian};
use structopt::StructOpt;
use serialport;
use serialport::SerialPort;

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
			eprintln!("Последовательный порт \"{}\" не найден.", opt.port);
			eprintln!("Список существующих:");
			for (i, p) in ports.iter().enumerate() {
				eprintln!("\t{}: {}", i, p.port_name);
			}
			process::exit(1);
		}
	};
	
	let mut port = serialport::new(port_name, opt.baudrate)
		.timeout(Duration::from_millis(1000))
		.parity(serialport::Parity::None)
		.open().expect("Не удалось открыть порт");
	
	let output = "This is a test. This is only a test.".as_bytes();
	port.write(output).expect("Ошибка записи данных в порт");

	let mut server = Server::new(port);
	server.start()?;
	
	Ok(())
}

struct Server {
	port:   Box<dyn SerialPort>,
	memory: Vec<u8>,
	query:  Vec<u8>,
}

impl Server {
	fn new(p: Box<dyn SerialPort>) -> Server {
		Server {
			port:   p,
			memory: vec![1u8; 1024], // TODO size
			query:  vec![0u8; 256], // TODO size
		}
	}

	fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		let mut pos: usize = 0;
		let mut ostream = BufWriter::new(self.port.try_clone()?);
		loop {
			match self.port.read(&mut self.query.as_mut_slice()[pos..]) {
				Err(e) => {
					println!("waiting, {}", e);
					pos = 0;
					continue;
				},
				Ok(n) => {
					println!("{} bytes received", n);
					if pos == 256 {
						eprintln!("Input buffer overflow");
						println!("{:?}", self.query);
						pos = 0;
						continue;
					}
					pos += n;
					if pos >= 2 {
						let slave_id  = self.query[0];
						let function = self.query[1];
						println!("slave id: {}", slave_id);
						let function_len = FUNC_LEN[function as usize];
						println!("function len: {}", function_len);
						// + 1 byte for slave id
						// + 2 bytes for crc
						if pos >= (function_len + 1 + 2) {
							println!("{:?}", self.query);
							ostream.write(&[slave_id]).unwrap();
							match self.process(pos) {
								Some(data) => { ostream.write_all(data.as_slice()).unwrap(); },
								None => {
									eprintln!("Invalid CRC, skipping query");
									pos = 0;
									continue;
								},
							}
							pos = 0;
						}
						else { continue; }
					}
					else { continue; }
				},
			}
			let crc_tx = self.crc(ostream.buffer());
			let crc_tx_buf = crc_tx.to_le_bytes();
			ostream.write_all(&crc_tx_buf).unwrap();
			// Write into serial port
			println!("{:?}", ostream.buffer());
			ostream.flush().unwrap();
		}
		Ok(())
	}

	fn process(&mut self, query_len: usize) -> Option<Vec<u8>> {
		let function  = self.query[1];
		let crc_rx: u16 = LittleEndian::read_u16(&self.query[query_len-2..query_len]);
		println!("function: {}", function);
		println!("received crc:   {}", crc_rx);

		// Check CRC
		let crc_calc = self.crc(&self.query[..query_len-2]);
		println!("calculated crc: {}", crc_calc);
		if crc_rx != crc_calc { return None; }
		
		match function {
			// Read coils
			0x01 => {
				println!("ReadCoils");
			},
			
			// Read discrete inputs
			0x02 => {
				println!("ReadDiscreteInputs");
			},
			
			// Read holding registers
			0x03 => {
				println!("ReadHoldingRegisters");
				let buf_start = BigEndian::read_u16(&self.query[2..4]);
				let buf_len   = BigEndian::read_u16(&self.query[4..6]);
				println!("offset:   {}", buf_start);
				println!("length:   {}", buf_len);
				let byte_count = buf_len * 2;
				
				let mut odat = Vec::with_capacity(64);
				odat.push(function);
				odat.push(byte_count as u8);
				odat.extend_from_slice(&self.memory[(buf_start as usize)..((buf_start+(buf_len * 2)) as usize)]);
				return Some(odat);
			},

			_ => {
				println!("Error: unknown modbus function code {}", function);
			}
		}
		
		Some(Vec::new())
	}

	fn crc(&self, buf: &[u8]) -> u16 {
		let mut crc: u16 = 0xFFFF;
		for &e in buf.iter() {
			crc ^= e as u16;             // XOR byte into least sig. byte of crc
			for _ in 0..8 {              // Loop over each bit
				if (crc & 0x0001) != 0 { // If the LSB is set
					crc >>= 1;           // Shift right and XOR 0xA001
					crc ^= 0xA001;
				}
				else {                   // Else LSB is not set
					crc >>= 1;           // Just shift right
				}
			}
		}
		// Note, this number has low and high bytes swapped, so use it accordingly (or swap bytes)
		crc
	}
}

// Length of queries (only data part) of different functions
// Max usize value in table means that size is dynamic
const FUNC_LEN: [usize; 0x30] = [
	0, //0x00
	5, //0x01 Read coils
	5, //0x02 Read discrete inputs
	5, //0x03 Read holding registers
	0, //0x04
	0, //0x05
	0, //0x06
	0, //0x07
	0, //0x08
	0, //0x09
	0, //0x0A
	0, //0x0B
	0, //0x0C
	0, //0x0D
	0, //0x0E
	0, //0x0F
	0, //0x10
	0, //0x11
	0, //0x12
	0, //0x13
	0, //0x14
	0, //0x15
	0, //0x16
	0, //0x17
	0, //0x18
	0, //0x19
	0, //0x1A
	0, //0x1B
	0, //0x1C
	0, //0x1D
	0, //0x1E
	0, //0x1F
	0, //0x20
	0, //0x21
	0, //0x22
	0, //0x23
	0, //0x24
	0, //0x25
	0, //0x26
	0, //0x27
	0, //0x28
	0, //0x29
	0, //0x2A
	0, //0x2B
	0, //0x2C
	0, //0x2D
	0, //0x2E
	0, //0x2F
];
