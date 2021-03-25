//------------------------------------------------------------------------------
// author:	Никита Макаревич, группа 3540901/02001
// email:	nikita.makarevich@spbpu.com
// 2021
//------------------------------------------------------------------------------
// Простой сервер Modbus RTU
// Структура сервера
//------------------------------------------------------------------------------
use std::io::BufWriter;
use std::io::Write;

use serialport;
use serialport::SerialPort;

mod formal;
use crate::server::formal::{ crc, get_func_len };

mod process;

pub struct Server {
	port:              Box<dyn SerialPort>,
	discrete_input:    Vec<u8>,
	coils:             Vec<u8>,
	input_registers:   Vec<u16>,
	holding_registers: Vec<u16>,
	query:             Vec<u8>,
}

pub const N_DISCRETE_INPUTS:   usize = 1024;
pub const N_COILS:             usize = 1024;
pub const N_INPUT_REGISTERS:   usize = 1024;
pub const N_HOLDING_REGISTERS: usize = 1024;
pub const IN_BUF_SIZE:         usize = 256;

impl Server {
	pub fn new(p: Box<dyn SerialPort>) -> Server {
		Server {
			port:              p,
			discrete_input:    vec![0; N_DISCRETE_INPUTS],
			coils:             vec![0; N_COILS],
			input_registers:   vec![0; N_INPUT_REGISTERS],
			holding_registers: vec![0; N_HOLDING_REGISTERS],
			query:             vec![0; IN_BUF_SIZE],
		}
	}

	pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		let mut pos: usize = 0;
		let mut ostream = BufWriter::new(self.port.try_clone()?);
		let mut func_len = IN_BUF_SIZE;
		
		loop {
			// TODO read with 'take'
			match self.port.read(&mut self.query.as_mut_slice()[pos..]) {
				Err(e) => {
					println!("Ожидание, {}", e);
					pos = 0;
					continue;
				},
				Ok(n) => {
					println!("{} байт получено", n);
					if pos == 0 { func_len = IN_BUF_SIZE; }
					if pos == IN_BUF_SIZE {
						eprintln!("Приёмный буфер переполнен");
						println!("{:?}", self.query);
						pos = 0;
						continue;
					}
					pos += n;
					if pos >= 2 {
						let slave_id = self.query[0];
						let function = self.query[1];
						println!("slave id: {}", slave_id); // DEBUG
						println!("function: {}", function); // DEBUG

						if func_len == IN_BUF_SIZE {
							match get_func_len(&self.query, pos.clone()) {
								Ok(usize::MAX) => func_len = IN_BUF_SIZE, // Недостаточно байт, чтобы понять длину
								Ok(l)          => func_len = l,           // Длина в байтах
								Err(what) => {                            // Ошибка
									eprintln!("Ошибка: {}. Запрос проигнорирован.", what);
									pos = 0;
									continue;
								},
							}
						}

						println!("func len: {}", func_len); // DEBUG
						// Проверка, набралась ли в буфере полная длина запроса
						// + 1 - slave id
						// + 2 - crc
						if pos >= (func_len + 1 + 2) {
							println!("{:?}", self.query[0..pos].to_vec());
							ostream.write(&[slave_id]).unwrap();
							match self.process(pos) {
								Ok(data) => { ostream.write_all(data.as_slice()).unwrap(); },
								Err(what) => {
									eprintln!("Ошибка: {}. Запрос проигнорирован.", what);
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
			let crc_tx = crc(ostream.buffer());
			ostream.write_all(&crc_tx.to_le_bytes()).unwrap();
			// Запись в последовательный порт
			println!("{:?}", ostream.buffer());
			ostream.flush().unwrap();
		}
		Ok(())
	}
}
