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
use crate::server::formal::{ crc, FUNC_LEN };

mod process;

pub struct Server {
	port:   Box<dyn SerialPort>,
	memory: Vec<u8>,
	query:  Vec<u8>,
}

impl Server {
	pub fn new(p: Box<dyn SerialPort>) -> Server {
		Server {
			port:   p,
			memory: vec![1u8; 1024],
			query:  vec![0u8; 256],
		}
	}

	pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		let mut pos: usize = 0;
		let mut ostream = BufWriter::new(self.port.try_clone()?);
		
		// Обработка PDU (Protocol data init)
		loop {
			match self.port.read(&mut self.query.as_mut_slice()[pos..]) {
				Err(e) => {
					println!("Ожидание, {}", e);
					pos = 0;
					continue;
				},
				Ok(n) => {
					println!("{} байт получено", n);
					if pos == 256 {
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
						let function_len = FUNC_LEN[function as usize];
						println!("function len: {}", function_len); // DEBUG
						// + 1 - slave id
						// + 2 - crc
						if pos >= (function_len + 1 + 2) {
							println!("{:?}", self.query);
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
