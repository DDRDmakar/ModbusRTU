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
	pub(super) fn process_function_code(&mut self) -> Result<Vec<u8>, MbExcWithMessage> {
		let function: u8 = self.query[1];
		let function_enum = num::FromPrimitive::from_u8(function);
		let mut odat = Vec::with_capacity(64);
		
		match function_enum { // TODO return error packets
			Some(MbFunc::ReadCoils) => {
				println!("ReadCoils");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				dbg!(offset);
				dbg!(quantity);

				if quantity == 0 || quantity > 2000 { return Err(MbExcWithMessage::new(MbExc::IllegalDataValue, STR_INVALID_QUANTITY.into())); }
				if offset + quantity >= N_COILS { return Err(MbExcWithMessage::new(MbExc::IllegalDataAddress, STR_INDEX_OUT.into())); }
				
				let n_bytes = (quantity as f32 / 8_f32).ceil() as usize;
				
				odat.push(n_bytes as u8);
				pack_bits(&self.coils[offset..offset + quantity], &mut odat);
				
				return Ok(odat);
			},
			
			Some(MbFunc::ReadDiscreteInputs) => {
				println!("ReadDiscreteInputs");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				dbg!(offset);
				dbg!(quantity);

				if quantity == 0 || quantity > 2000 { return Err(MbExcWithMessage::new(MbExc::IllegalDataValue, STR_INVALID_QUANTITY.into())); }
				if offset + quantity >= N_DISCRETE_INPUTS { return Err(MbExcWithMessage::new(MbExc::IllegalDataAddress, STR_INDEX_OUT.into())); }
				
				let n_bytes = (quantity as f32 / 8_f32).ceil() as usize;
				
				odat.push(n_bytes as u8);
				pack_bits(&self.discrete_input[offset..offset + quantity], &mut odat);
				
				return Ok(odat);
			},
			
			Some(MbFunc::ReadHoldingRegisters) => {
				println!("ReadHoldingRegisters");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				dbg!(offset);
				dbg!(quantity);
				let byte_count = quantity * 2;

				if quantity == 0 || quantity > 125 { return Err(MbExcWithMessage::new(MbExc::IllegalDataValue, STR_INVALID_QUANTITY.into())); }
				if offset + quantity >= N_HOLDING_REGISTERS { return Err(MbExcWithMessage::new(MbExc::IllegalDataAddress, STR_INDEX_OUT.into())); }
				
				odat.push(byte_count as u8);
				let tlen = odat.len();
				odat.resize(tlen + byte_count, 0);
				BigEndian::write_u16_into(
					&self.holding_registers[offset..offset + quantity],
					&mut odat[tlen..]
				);
				return Ok(odat);
			},

			Some(MbFunc::ReadInputRegisters) => {
				println!("ReadInputRegisters");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				dbg!(offset);
				dbg!(quantity);
				let byte_count = quantity * 2;

				if quantity == 0 || quantity > 125 { return Err(MbExcWithMessage::new(MbExc::IllegalDataValue, STR_INVALID_QUANTITY.into())); }
				if offset + quantity >= N_INPUT_REGISTERS { return Err(MbExcWithMessage::new(MbExc::IllegalDataAddress, STR_INDEX_OUT.into())); }
				
				odat.push(byte_count as u8);
				let tlen = odat.len();
				odat.resize(tlen + byte_count, 0);
				BigEndian::write_u16_into(
					&self.input_registers[offset..offset + quantity],
					&mut odat[tlen..]
				);
				return Ok(odat);
			},

			Some(MbFunc::WriteSingleCoil) => {
				println!("WriteSingleCoil");
				let offset = BigEndian::read_u16(&self.query[2..4]) as usize;
				let value = BigEndian::read_u16(&self.query[4..6]);
				dbg!(offset);
				dbg!(value);

				if offset >= N_COILS { return Err(MbExcWithMessage::new(MbExc::IllegalDataAddress, STR_INDEX_OUT.into())); }
				if value != 0x0000 && value != 0xFF00 { return Err(MbExcWithMessage::new(MbExc::IllegalDataValue, "Недействительное значение coil".into())); }

				self.coils[offset] = if value == 0 { 0 } else { 1 };
				odat.extend(&(offset as u16).to_be_bytes());
				odat.extend(&(value).to_be_bytes());
				return Ok(odat);
			},

			Some(MbFunc::WriteSingleRegister) => {
				println!("WriteSingleRegister");
				let offset = BigEndian::read_u16(&self.query[2..4]) as usize;
				let value = BigEndian::read_u16(&self.query[4..6]);
				dbg!(offset);
				dbg!(value);

				if offset >= N_HOLDING_REGISTERS { return Err(MbExcWithMessage::new(MbExc::IllegalDataAddress, STR_INDEX_OUT.into())); }

				self.holding_registers[offset] = value;
				odat.extend(&(offset as u16).to_be_bytes());
				odat.extend(&(value as u16).to_be_bytes());
				return Ok(odat);
			},

			Some(MbFunc::WriteMultipleCoils) => {
				println!("WriteMultipleCoils");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				dbg!(offset);
				dbg!(quantity);
				let byte_count = self.query[6] as usize;
				let byte_count_from_quantity = (quantity as f32 / 8_f32).ceil() as usize;
				
				if quantity == 0 || quantity > 0x07B0 { return Err(MbExcWithMessage::new(MbExc::IllegalDataValue, STR_INVALID_QUANTITY.into())); }
				if byte_count != byte_count_from_quantity { return Err(MbExcWithMessage::new(MbExc::IllegalDataValue, STR_INVALID_BYTE_COUNT.into())); }
				if offset + quantity >= N_COILS { return Err(MbExcWithMessage::new(MbExc::IllegalDataAddress, STR_INDEX_OUT.into())); }

				unpack_bits(&self.query[7..7+byte_count], &mut self.coils[offset..offset+quantity]);
				odat.extend(&(offset as u16).to_be_bytes());
				odat.extend(&(quantity as u16).to_be_bytes());
				return Ok(odat);
			},
			
			Some(MbFunc::WriteMultipleRegisters) => {
				println!("WriteMultipleRegisters");
				let offset    = BigEndian::read_u16(&self.query[2..4]) as usize;
				let quantity  = BigEndian::read_u16(&self.query[4..6]) as usize;
				dbg!(offset);
				dbg!(quantity);
				let byte_count = self.query[6] as usize;

				if quantity == 0 || quantity > 0x007B { return Err(MbExcWithMessage::new(MbExc::IllegalDataValue, STR_INVALID_QUANTITY.into())); }
				if byte_count != quantity * 2 { return Err(MbExcWithMessage::new(MbExc::IllegalDataValue, STR_INVALID_BYTE_COUNT.into())); }
				if offset + quantity >= N_HOLDING_REGISTERS { return Err(MbExcWithMessage::new(MbExc::IllegalDataAddress, STR_INDEX_OUT.into())); }
				
				odat.extend(&(offset as u16).to_be_bytes());
				odat.extend(&(quantity as u16).to_be_bytes());
				BigEndian::read_u16_into(
					&self.query[7..7 + byte_count],
					&mut self.holding_registers[offset..offset + quantity]
				);
				return Ok(odat);
			},

			None => Err(MbExcWithMessage::new(MbExc::IllegalFunction, STR_ILLEGAL_FUNCTION.into())),
			
		} // End match
	} // End fn
} // End impl
