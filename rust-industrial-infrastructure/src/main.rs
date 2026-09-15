// ============================================================================
// Memory-Safe Industrial Infrastructure Platform
// Language: Rust 1.70+
// ============================================================================

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ModbusFunction {
    ReadHoldingRegisters = 0x03,
    WriteSingleRegister = 0x06,
    WriteMultipleRegisters = 0x10,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModbusTcpFrame<'a> {
    pub transaction_id: u16,
    pub protocol_id: u16,
    pub length: u16,
    pub unit_id: u8,
    pub function_code: u8,
    pub payload: &'a [u8],
}

impl<'a> ModbusTcpFrame<'a> {
    /// Zero-copy parsing of Modbus MBAP header and PDU payload
    pub fn parse(buffer: &'a [u8]) -> Result<Self, &'static str> {
        if buffer.len() < 8 {
            return Err("Packet too short for MBAP header");
        }

        let transaction_id = u16::from_be_bytes([buffer[0], buffer[1]]);
        let protocol_id = u16::from_be_bytes([buffer[2], buffer[3]]);
        let length = u16::from_be_bytes([buffer[4], buffer[5]]);
        let unit_id = buffer[6];
        let function_code = buffer[7];

        if protocol_id != 0 {
            return Err("Invalid protocol identifier (expected 0 for Modbus)");
        }

        let payload_len = (length as usize).saturating_sub(2);
        if buffer.len() < 8 + payload_len {
            return Err("Truncated payload");
        }

        let payload = &buffer[8..8 + payload_len];

        Ok(ModbusTcpFrame {
            transaction_id,
            protocol_id,
            length,
            unit_id,
            function_code,
            payload,
        })
    }
}

pub fn calculate_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if (crc & 0x0001) != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

fn main() {
    println!("=== Memory-Safe Industrial Infrastructure Platform (Rust) ===");
    
    // Test Modbus TCP Frame: Tx=101, Proto=0, Len=6, Unit=1, Func=0x03 (Read), Addr=0x0064, Qty=0x0002
    let packet: [u8; 12] = [
        0x00, 0x65, // Transaction ID: 101
        0x00, 0x00, // Protocol ID: 0 (Modbus)
        0x00, 0x06, // Length: 6 bytes following
        0x01,       // Unit ID: 1
        0x03,       // Function Code: 0x03 (Read Holding Registers)
        0x00, 0x64, // Starting Register: 100
        0x00, 0x02, // Register Count: 2
    ];

    match ModbusTcpFrame::parse(&packet) {
        Ok(frame) => {
            println!("[ZERO-COPY PARSER] Decoded Modbus TCP Frame:");
            println!("  Transaction ID: {}", frame.transaction_id);
            println!("  Unit ID       : {}", frame.unit_id);
            println!("  Function Code : 0x{:02X}", frame.function_code);
            println!("  Payload Bytes : {:?}", frame.payload);
        }
        Err(e) => panic!("Parse failed: {}", e),
    }

    let crc = calculate_crc16(&packet[6..]);
    println!("[CRC16-MODBUS] Checksum: 0x{:04X}", crc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_modbus_frame_parsing() {
        let packet: [u8; 12] = [
            0x00, 0x01, // Tx ID: 1
            0x00, 0x00, // Proto: 0 (Modbus)
            0x00, 0x06, // Len: 6
            0x01,       // Unit: 1
            0x03,       // Function: Read
            0x00, 0x00, 0x00, 0x0A
        ];

        let frame = ModbusTcpFrame::parse(&packet).expect("Valid frame should parse");
        assert_eq!(frame.transaction_id, 1);
        assert_eq!(frame.protocol_id, 0);
        assert_eq!(frame.unit_id, 1);
        assert_eq!(frame.function_code, 0x03);
        assert_eq!(frame.payload, &[0x00, 0x00, 0x00, 0x0A]);
    }

    #[test]
    fn test_truncated_packet_error() {
        let short_packet: [u8; 4] = [0x00, 0x01, 0x00, 0x00];
        assert!(ModbusTcpFrame::parse(&short_packet).is_err());
    }

    #[test]
    fn test_invalid_protocol_id() {
        let invalid_proto: [u8; 12] = [
            0x00, 0x01,
            0x00, 0x01, // Proto 1 != 0
            0x00, 0x06,
            0x01, 0x03, 0x00, 0x00, 0x00, 0x0A
        ];
        assert!(ModbusTcpFrame::parse(&invalid_proto).is_err());
    }

    #[test]
    fn test_crc16_calculation() {
        let test_bytes = [0x01, 0x03, 0x00, 0x64, 0x00, 0x02];
        let crc = calculate_crc16(&test_bytes);
        assert_ne!(crc, 0);
    }
}
