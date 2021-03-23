//------------------------------------------------------------------------------
// author:	Nikita Makarevich
// email:	nikita.makarevich@spbpu.com
// 2021
//------------------------------------------------------------------------------
// Simple Modbus RTU server
// Processing of query PDU (Protocol data init)
//------------------------------------------------------------------------------
use byteorder::{ ByteOrder, BigEndian, LittleEndian };

use crate::server::formal::{ crc };
use crate::server::Server;

impl Server {
	pub(super) fn process(&mut self, query_len: usize) -> Result<Vec<u8>, &'static str> {
		let function  = self.query[1];
		let crc_rx: u16 = LittleEndian::read_u16(&self.query[query_len-2..query_len]);
		println!("function: {}", function);
		println!("received crc:   {}", crc_rx);

		// Check CRC
		let crc_calc = crc(&self.query[..query_len-2]);
		println!("calculated crc: {}", crc_calc);
		if crc_rx != crc_calc { return Err("Invalid CRC"); }
		
		match function {
			// Read coils
			0x01 => {
				println!("ReadCoils");
				Err("Not implemented yet")
			},
			
			// Read discrete inputs
			0x02 => {
				println!("ReadDiscreteInputs");
				Err("Not implemented yet")
			},
			
			// Read holding registers
			0x03 => {
				println!("ReadHoldingRegisters");
				let buf_start = BigEndian::read_u16(&self.query[2..4]);
				let quantity  = BigEndian::read_u16(&self.query[4..6]);
				println!("offset:   {}", buf_start);
				println!("quantity: {}", quantity);
				let byte_count = quantity * 2;

				if quantity == 0 || quantity > 125 { return Err("Invalid registers quantity"); }
				
				let mut odat = Vec::with_capacity(64);
				odat.push(function);
				odat.push(byte_count as u8);
				odat.extend_from_slice(&self.memory[(buf_start as usize)..((buf_start + byte_count) as usize)]);
				return Ok(odat)
			},

			// Write multiple registers
			0x10 => {
				println!("WriteMultipleRegisters");
				let buf_start = BigEndian::read_u16(&self.query[2..4]);
				let quantity  = BigEndian::read_u16(&self.query[4..6]);
				println!("offset:   {}", buf_start);
				println!("quantity: {}", quantity);
				let byte_count = self.query[6];

				if quantity == 0 || quantity > 123 { return Err("Invalid registers quantity"); }				
				if byte_count as u16 != quantity * 2      { return Err("Invalid byte count"); }
				
				let mut odat = Vec::with_capacity(64);
				odat.push(function);
				odat.extend_from_slice(&buf_start.to_be_bytes());
				odat.extend_from_slice(&quantity.to_be_bytes());
				let registers = &self.query[7..(7 + byte_count as usize)];
				for (i, &r) in registers.iter().enumerate() {
					self.memory[(buf_start as usize) + i] = r;
				}
				return Ok(odat)
			},

			_ => { Err("Unknown modbus function code") }
			
		} // End match
	} // End fn
} // End impl
