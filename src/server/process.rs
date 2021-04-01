//------------------------------------------------------------------------------
// author:	Nikita Makarevich (aka DDRDmakar)
// email:	makarevich.98@mail.ru
// 2021
// This code is under MIT license (see LICENSE.txt)
//------------------------------------------------------------------------------
// Simple Modbus RTU server
// Processing of query PDU (Protocol data init)
//------------------------------------------------------------------------------
use byteorder::{ ByteOrder, BigEndian };

use crate::server::Server;
use crate::server::{ N_DISCRETE_INPUTS, N_COILS, N_INPUT_REGISTERS, N_HOLDING_REGISTERS };
use crate::server::formal::*;

impl Server {
	pub(super) fn process_function_code(&mut self) -> Result<Vec<u8>, IntErrWithMessage> {
		let function: u8 = self.query[1];
		let function_enum = num::FromPrimitive::from_u8(function);
		match function_enum { // TODO return error packets
			Some(MbFunc::ReadCoils) => {
				println!("ReadCoils");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				dbg!(offset);
				dbg!(quantity);

				if quantity == 0 || quantity > 2000 { return Err(int_err(IntErr::InvalidQueryParameter, "Invalid quantity".into())); }
				if offset + quantity >= N_COILS { return Err(int_err(IntErr::InvalidQueryParameter, "Index out of bounds".into())); }
				
				let n_bytes = (quantity as f32 / 8_f32).ceil() as usize;
				
				let mut odat = Vec::with_capacity(64);
				odat.push(n_bytes as u8);
				pack_bits(&self.coils[offset..offset + quantity], &mut odat);
				
				return Ok(odat)
			},
			
			Some(MbFunc::ReadDiscreteInputs) => {
				println!("ReadDiscreteInputs");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				dbg!(offset);
				dbg!(quantity);

				if quantity == 0 || quantity > 2000 { return Err(int_err(IntErr::InvalidQueryParameter, "Invalid quantity".into())); }
				if offset + quantity >= N_DISCRETE_INPUTS { return Err(int_err(IntErr::InvalidQueryParameter, "Index out of bounds".into())); }
				
				let n_bytes = (quantity as f32 / 8_f32).ceil() as usize;
				
				let mut odat = Vec::with_capacity(64);
				odat.push(n_bytes as u8);
				pack_bits(&self.discrete_input[offset..offset + quantity], &mut odat);
				
				return Ok(odat)
			},
			
			Some(MbFunc::ReadHoldingRegisters) => {
				println!("ReadHoldingRegisters");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				dbg!(offset);
				dbg!(quantity);
				let byte_count = quantity * 2;

				if quantity == 0 || quantity > 125 { return Err(int_err(IntErr::InvalidQueryParameter, "Invalid quantity".into())); }
				if offset + quantity >= N_HOLDING_REGISTERS { return Err(int_err(IntErr::InvalidQueryParameter, "Index out of bounds".into())); }
				
				let mut odat = Vec::with_capacity(64);
				odat.push(byte_count as u8);
				let tlen = odat.len();
				odat.resize(tlen + byte_count, 0);
				BigEndian::write_u16_into(
					&self.holding_registers[offset..offset + quantity],
					&mut odat[tlen..]
				);
				return Ok(odat)
			},

			Some(MbFunc::ReadInputRegisters) => {
				println!("ReadInputRegisters");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				dbg!(offset);
				dbg!(quantity);
				let byte_count = quantity * 2;

				if quantity == 0 || quantity > 125 { return Err(int_err(IntErr::InvalidQueryParameter, "Invalid quantity".into())); }
				if offset + quantity >= N_INPUT_REGISTERS { return Err(int_err(IntErr::InvalidQueryParameter, "Index out of bounds".into())); }
				
				let mut odat = Vec::with_capacity(64);
				odat.push(byte_count as u8);
				let tlen = odat.len();
				odat.resize(tlen + byte_count, 0);
				BigEndian::write_u16_into(
					&self.input_registers[offset..offset + quantity],
					&mut odat[tlen..]
				);
				return Ok(odat)
			},

			Some(MbFunc::WriteMultipleRegisters) => {
				println!("WriteMultipleRegisters");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				dbg!(offset);
				dbg!(quantity);
				let byte_count = self.query[6] as usize;

				if quantity == 0 || quantity > 123 { return Err(int_err(IntErr::InvalidQueryParameter, "Invalid quantity".into())); }
				if byte_count != quantity * 2 { return Err(int_err(IntErr::InvalidQueryParameter, "Byte count does not match quantity".into())); }
				if offset + quantity >= N_HOLDING_REGISTERS { return Err(int_err(IntErr::InvalidQueryParameter, "Index out of bounds".into())); }
				
				let mut odat = Vec::with_capacity(64);
				odat.extend(&(offset as u16).to_be_bytes());
				odat.extend(&(quantity as u16).to_be_bytes());
				BigEndian::read_u16_into(
					&self.query[7..7 + byte_count],
					&mut self.holding_registers[offset..offset + quantity]
				);
				return Ok(odat)
			},

			None => { Err(int_err(IntErr::UnknownFunctionCode, "Unknown modbus function code".into())) }
			
		} // End match
	} // End fn
} // End impl
