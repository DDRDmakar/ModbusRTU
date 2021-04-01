//------------------------------------------------------------------------------
// author:	Nikita Makarevich (aka DDRDmakar)
// email:	makarevich.98@mail.ru
// 2021
// This code is under MIT license (see LICENSE.txt)
//------------------------------------------------------------------------------
// Простой сервер Modbus RTU
// Структура сервера
//------------------------------------------------------------------------------
use std::io::BufWriter;
use std::io::Write;
use std::time::Duration;
use std::thread;

use serialport;
use serialport::{ SerialPort, Parity, StopBits };
use byteorder::{ ByteOrder, LittleEndian };

mod formal;
use crate::server::formal::*;
mod process;

pub struct Server {
	port:              Box<dyn SerialPort>,
	discrete_input:    Vec<u8>,
	coils:             Vec<u8>,
	input_registers:   Vec<u16>,
	holding_registers: Vec<u16>,
	query:             Vec<u8>,
	pos:               usize,
	query_len:         usize,
	//ostream:           &'a BufWriter<dyn SerialPort>,
	response_delay:    Duration,
}

pub const N_DISCRETE_INPUTS:   usize = 1024;
pub const N_COILS:             usize = 1024;
pub const N_INPUT_REGISTERS:   usize = 1024;
pub const N_HOLDING_REGISTERS: usize = 1024;
pub const IN_BUF_SIZE:         usize = 256;

impl Server {
	pub fn new(p: Box<dyn SerialPort>) -> Server {
		let us_per_bit = 1000000f32 / p.baud_rate().unwrap() as f32;
		let n_parity_bits = match p.parity().unwrap() {
			Parity::None => 0,
			Parity::Odd  => 1,
			Parity::Even => 1,
		};
		let n_stop_bits = match p.stop_bits().unwrap() {
			StopBits::One => 1,
			StopBits::Two => 2,
		};
		let n_bits_per_symbol = 1 + 8 + n_parity_bits + n_stop_bits;
		let us_per_symbol = us_per_bit * n_bits_per_symbol as f32;
		dbg!(us_per_bit);
		dbg!(n_bits_per_symbol);
		dbg!(us_per_symbol);
		
		Server {
			port:              p,
			discrete_input:    vec![0; N_DISCRETE_INPUTS],
			coils:             vec![0; N_COILS],
			input_registers:   vec![0; N_INPUT_REGISTERS],
			holding_registers: vec![0; N_HOLDING_REGISTERS],
			query:             vec![0; IN_BUF_SIZE],
			//ostream:           &BufWriter::new(p.try_clone()?),
			query_len:         usize::MAX, // Недостаточно данных, чтобы определить длину пакета
			pos:               0,
			response_delay:    Duration::from_micros((us_per_symbol * 4.0) as u64),
		}
	}

	#[allow(unreachable_code)]
	pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		let mut ostream = BufWriter::new(self.port.try_clone()?);
		loop {
			let pos_read_to = if self.query_len == usize::MAX { self.pos + 1 } else { self.query_len };
			match self.port.read(&mut self.query[self.pos..pos_read_to]) {
				Err(e) => {
					println!("Ожидание, {}", e);
					if self.pos != 0 { println!("RX {:02X?}", &self.query[..self.pos]); }
					self.pos = 0;
					continue;
				},
				Ok(n) => {
					println!("{} байт получено", n);
					if self.pos == 0 { self.query_len = usize::MAX; }
					self.pos += n;
					if self.pos >= 2 {
						let slave_id = self.query[0];
						let function = self.query[1];
						dbg!(slave_id);
						dbg!(function);

						// Определение длины сообщения
						if self.query_len == usize::MAX {
							match self.get_query_len() {
								Ok(l) => self.query_len = l,
								Err(IntErrWithMessage { err: _, message }) => {
									// TODO error handling
									eprintln!("Ошибка: {}. Запрос проигнорирован.", message);
									self.pos = 0;
									continue;
									//match err {
									//	IntErr::UnknownFunctionCode => {
									//	},
									//	IntErr::WrongBranch => {
									//	},
									//	IntErr::InvalidQueryParameter => {
									//	},
									//}
								},
							}
						}
						dbg!(self.query_len);
						
						if self.pos >= self.query_len {
							println!("RX {:02X?}", &self.query[..self.query_len]);

							// Check CRC
							let crc_rx: u16 = LittleEndian::read_u16(&self.query[self.query_len - 2..self.query_len]);
							let crc_calc = crc(&self.query[..self.query_len - 2]);
							dbg!(crc_rx);
							dbg!(crc_calc);
							if crc_rx != crc_calc {
								eprintln!("Ошибка CRC. Запрос проигнорирован.");
								self.pos = 0;
								continue;
							}
							
							ostream.write(&[slave_id, function]).unwrap();
							match self.process_function_code() {
								Ok(data) => { ostream.write(data.as_slice()).unwrap(); },
								Err(IntErrWithMessage { err: _, message }) => {
									// TODO error handling
									eprintln!("Ошибка: {}. Запрос проигнорирован.", message);
								},
							}
						}
						else { continue; }
					}
					else { continue; }
				},
			}
			let crc_tx = crc(ostream.buffer());
			ostream.write(&crc_tx.to_le_bytes()).unwrap();
			println!("TX {:02X?}", ostream.buffer());
			thread::sleep(self.response_delay);
			// Запись в последовательный порт
			ostream.flush().unwrap();
			self.pos = 0;
			continue;
		}
		Ok(())
	}
	
	// Вычисление длины запроса, если её не получается определить по коду функции
	// Здесь к длине прибавляется 3 (+1+2)
	// +1 - длина device id
	// +2 - длина CRC
	// Возвращает Ok(usize::MAX), если длину пока определить нельзя
	fn get_query_len(&self) -> Result<usize, IntErrWithMessage> {
		const STR_UNKNOWN_F: &str = "Неизвестный код функции";
		if self.pos < 2 { return Ok(usize::MAX); }
		let function: u8 = self.query[1];
		if (function as usize) < QUERY_LEN.len() {
			match QUERY_LEN[function as usize] {
				usize::MAX => {
					let function_enum = num::FromPrimitive::from_u8(function);
					let answer = match function_enum {
						Some(MbFunc::WriteMultipleRegisters) => {
							if self.pos > 6 { Ok((self.query[6] + 6 + 1 + 2) as usize) }
							else { Ok(usize::MAX) }
						},
						Some(_) => Err(int_err(IntErr::WrongBranch,"Попытка вычислить длину сообщения со статической длиной".into())),
						None => Err(int_err(IntErr::UnknownFunctionCode, STR_UNKNOWN_F.into())),
					};
					match answer {
						Ok(l) => {
							if l > IN_BUF_SIZE { Err(int_err(IntErr::InvalidQueryParameter, "Неверная длина пакета".into())) }
							else { answer }
						},
						_ => answer,
					}
				},
				0 => Err(int_err(IntErr::UnknownFunctionCode, STR_UNKNOWN_F.into())),
				fixed => Ok(fixed + 1 + 2),
			}
		} else { Err(int_err(IntErr::UnknownFunctionCode, STR_UNKNOWN_F.into())) }
	}
}
