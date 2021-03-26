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
use crate::server:: { N_DISCRETE_INPUTS, N_COILS, N_INPUT_REGISTERS, N_HOLDING_REGISTERS, IN_BUF_SIZE };

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
		
		match function { // TODO return error packets
			// Read coils
			0x01 => {
				println!("ReadCoils");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				println!("offset:   {}", offset);
				println!("quantity: {}", quantity);

				if quantity == 0 || quantity > 2000 { return Err("Invalid quantity"); }
				if offset + quantity >= N_COILS { return Err("Index out of bounds"); }
				
				let n_bytes = (quantity as f32 / 8_f32).ceil() as usize;
				
				let mut odat = Vec::with_capacity(64);
				odat.push(function);
				odat.push(n_bytes as u8);
				self.pack_bits(&self.coils[offset..offset + quantity], &mut odat);
				
				return Ok(odat)
			},
			
			// Read discrete inputs
			0x02 => {
				println!("ReadDiscreteInputs");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				println!("offset:   {}", offset);
				println!("quantity: {}", quantity);

				if quantity == 0 || quantity > 2000 { return Err("Invalid quantity"); }
				if offset + quantity >= N_DISCRETE_INPUTS { return Err("Index out of bounds"); }
				
				let n_bytes = (quantity as f32 / 8_f32).ceil() as usize;
				
				let mut odat = Vec::with_capacity(64);
				odat.push(function);
				odat.push(n_bytes as u8);
				self.pack_bits(&self.discrete_input[offset..offset + quantity], &mut odat);
				
				return Ok(odat)
			},
			
			// Read holding registers
			0x03 => {
				println!("ReadHoldingRegisters");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				println!("offset:   {}", offset);
				println!("quantity: {}", quantity);
				let byte_count = quantity * 2;

				if quantity == 0 || quantity > 125 { return Err("Invalid registers quantity"); }
				if offset + quantity >= N_HOLDING_REGISTERS { return Err("Index out of bounds"); }
				
				let mut odat = Vec::with_capacity(64);
				odat.push(function);
				odat.push(byte_count as u8);
				let tlen = odat.len();
				odat.resize(tlen + byte_count, 0);
				BigEndian::write_u16_into(
					&self.holding_registers[offset..offset + quantity],
					&mut odat[tlen..]
				);
				return Ok(odat)
			},

			// Read input registers
			0x04 => {
				println!("ReadInputRegisters");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				println!("offset:   {}", offset);
				println!("quantity: {}", quantity);
				let byte_count = quantity * 2;

				if quantity == 0 || quantity > 125 { return Err("Invalid registers quantity"); }
				if offset + quantity >= N_INPUT_REGISTERS { return Err("Index out of bounds"); }
				
				let mut odat = Vec::with_capacity(64);
				odat.push(function);
				odat.push(byte_count as u8);
				let tlen = odat.len();
				odat.resize(tlen + byte_count, 0);
				BigEndian::write_u16_into(
					&self.input_registers[offset..offset + quantity],
					&mut odat[tlen..]
				);
				return Ok(odat)
			},

			// Write multiple registers
			0x10 => {
				println!("WriteMultipleRegisters");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				println!("offset:   {}", offset);
				println!("quantity: {}", quantity);
				let byte_count = self.query[6] as usize;

				if quantity == 0 || quantity > 123 { return Err("Invalid registers quantity"); }				
				if byte_count != quantity * 2 { return Err("Invalid byte count"); }
				if offset + quantity >= N_HOLDING_REGISTERS { return Err("Index out of bounds"); }
				
				let mut odat = Vec::with_capacity(64);
				odat.push(function);
				odat.extend(&(offset as u16).to_be_bytes());
				odat.extend(&(quantity as u16).to_be_bytes());
				BigEndian::read_u16_into(
					&self.query[7..7 + byte_count],
					&mut self.holding_registers[offset..offset + quantity]
				);
				return Ok(odat)
			},

			_ => { Err("Unknown modbus function code") }
			
		} // End match
	} // End fn

	fn pack_bits(&self, src: &[u8], dst: &mut Vec<u8>) {
		let mut val: u8 = 0;
		for (i, &e) in src.iter().enumerate() {
			let mod8 = i % 8;
			val |= e << mod8;
			if mod8 == 7 {
				dst.push(val);
				val = 0u8;
			}
		}
		if src.len() % 8 != 0 { dst.push(val); }
	}
	
} // End impl
