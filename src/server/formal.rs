//------------------------------------------------------------------------------
// author:	Nikita Makarevich (aka DDRDmakar)
// email:	makarevich.98@mail.ru
// 2021
// This code is under MIT license (see LICENSE.txt)
//------------------------------------------------------------------------------
// Простой сервер Modbus RTU
// Формальные части программы
//------------------------------------------------------------------------------

// Расчёт CRC по спецификации Modbus
pub fn crc(buf: &[u8]) -> u16 {
	let mut crc: u16 = 0xFFFF;
	for &e in buf.iter() {
		crc ^= e as u16;             // XOR байта с младшим байтом CRC
		for _ in 0..8 {              // Итерируемся по всем битам
			if (crc & 0x0001) != 0 { // Если LSB == 1
				crc >>= 1;           // Сдвиг вправо и XOR с 0xA001
				crc ^= 0xA001;
			}
			else {                   // Иначе, если LSB == 0
				crc >>= 1;           // Только сдвиг вправо
			}
		}
	}
	// Внимание! Порядок байтов в crc может не сответствовать Modbus
	crc
}

// Упаковка байтов в биты для передачи через Modbus
pub fn pack_bits(src: &[u8], dst: &mut Vec<u8>) {
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

// Распаковка битов, принятых через Modbus, в массив байтов
pub fn unpack_bits(src: &[u8], dst: &mut Vec<u8>) {
	for e in src.iter() {
		let mut mask = 0x80;
		for _ in 0..8 {
			let val = if e & mask == 0 { 0 } else { 1 };
			dst.push(val);
			mask >>= 1;
		}
	}
}

// Modbus function codes
#[derive(FromPrimitive)]
pub enum MbFunc {
	ReadCoils              = 0x01,
	ReadDiscreteInputs     = 0x02,
	ReadHoldingRegisters   = 0x03,
	ReadInputRegisters     = 0x04,
	WriteSingleCoil        = 0x05,
	WriteSingleRegister    = 0x06,
	WriteMultipleRegisters = 0x10,
}

// Modbus exception codes
#[repr(u8)]
#[derive(FromPrimitive)]
pub enum MbExc {
	IllegalFunction    = 1,
	IllegalDataAddress = 2,
	IllegalDataValue   = 3,
	SlaveDeviceFailure = 4,
	Acknowledge        = 5,
	SlaveDeviceBusy    = 6,
	MemoryParityError  = 8,
	GatewayPathUnavailable = 0xA,
	GatewayTargetDeviceFailedToRespond = 0xB,
}

pub struct MbExcWithMessage {
	pub exc: MbExc,
	pub message: String,
}

impl MbExcWithMessage {
	pub fn new(exc: MbExc, message: String) -> MbExcWithMessage {
		MbExcWithMessage {
			exc: exc,
			message: message,
		}
	}
}

pub const STR_ILLEGAL_FUNCTION: &str = "Недействительный код функции";
pub const STR_INVALID_QUANTITY: &str = "Неверное количество байт (quantity)";
pub const STR_INDEX_OUT: &str = "Адрес выходит за допустимые пределы";
pub const STR_INVALID_BYTE_COUNT: &str = "Значение \"byte count\" не соответствует значению \"quantity\"";

// Длина области данных для различных функций Modbus RTU.
// usize::MAX - Размер вычисляется динамически.
// 0 - Несуществующие функции.
pub const QUERY_LEN: [usize; 0x30] = [
	0, // 0x00
	5, // 0x01 Read coils
	5, // 0x02 Read discrete inputs
	5, // 0x03 Read holding registers
	5, // 0x04 Read input registers
	5, // 0x05 Write single coil
	5, // 0x06 Write single register
	0, // 0x07
	0, // 0x08
	0, // 0x09
	0, // 0x0A
	0, // 0x0B
	0, // 0x0C
	0, // 0x0D
	0, // 0x0E
	0, // 0x0F
	usize::MAX, // 0x10 Write multiple registers
	0, // 0x11
	0, // 0x12
	0, // 0x13
	0, // 0x14
	0, // 0x15
	0, // 0x16
	0, // 0x17
	0, // 0x18
	0, // 0x19
	0, // 0x1A
	0, // 0x1B
	0, // 0x1C
	0, // 0x1D
	0, // 0x1E
	0, // 0x1F
	0, // 0x20
	0, // 0x21
	0, // 0x22
	0, // 0x23
	0, // 0x24
	0, // 0x25
	0, // 0x26
	0, // 0x27
	0, // 0x28
	0, // 0x29
	0, // 0x2A
	0, // 0x2B
	0, // 0x2C
	0, // 0x2D
	0, // 0x2E
	0, // 0x2F
];

