//------------------------------------------------------------------------------
// author:	Никита Макаревич, группа 3540901/02001
// email:	nikita.makarevich@spbpu.com
// 2021
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
	// Внимание! Порядок байтов в crc - little-endian
	crc
}

const STR_UNKNOWN_F: &str = "Неизвестный код функции";

pub fn get_func_len(buf: &Vec<u8>, pos: usize) -> Result<usize, &'static str> {
	if pos < 2 { return Ok(usize::MAX); }
	let function = buf[1];
	if function < 0x30 {
		match FUNC_LEN[function as usize] {
			usize::MAX => {
				match function {
					0x10 => { // Write multiple registers
						if pos > 4 { Ok(buf[4] as usize) }
						else       { Ok(usize::MAX) }
					},
					_ => Err(STR_UNKNOWN_F),
				}
			},
			0 => Err(STR_UNKNOWN_F),
			fixed => Ok(fixed),
		}
	} else { Err(STR_UNKNOWN_F) }
}

// Длина области данных для различных функций Modbus RTU.
// usize::MAX - Размер вычисляется динамически.
// 0 - Несуществующие функции.
const FUNC_LEN: [usize; 0x30] = [
	0, // 0x00
	5, // 0x01 Read coils
	5, // 0x02 Read discrete inputs
	5, // 0x03 Read holding registers
	0, // 0x04
	0, // 0x05
	0, // 0x06
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
