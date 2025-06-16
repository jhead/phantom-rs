use bytes::{Buf, BufMut, Bytes, BytesMut};

// Packet constants
pub const UNCONNECTED_PING_ID: u8 = 0x01;

// Magic bytes used in the protocol
pub const MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

#[derive(Debug, Clone)]
pub struct UnconnectedPing {
    pub ping_time: [u8; 8],
    pub magic: [u8; 16],
    pub client_id: [u8; 8],
}

impl UnconnectedPing {
    /// Creates a new UnconnectedPing with default values
    pub fn new(client_id: [u8; 8], ping_time: [u8; 8]) -> Self {
        Self {
            ping_time,
            magic: MAGIC,
            client_id,
        }
    }

    /// Serializes the UnconnectedPing into bytes for the 0x01 packet
    pub fn build(&self) -> Bytes {
        let mut buf = BytesMut::new();

        // Packet ID
        buf.put_u8(UNCONNECTED_PING_ID);

        // Ping time (8 bytes)
        buf.put_slice(&self.ping_time);

        // Magic (16 bytes)
        buf.put_slice(&self.magic);

        // Client ID (8 bytes)
        buf.put_slice(&self.client_id);

        buf.freeze()
    }

    /// Deserializes an UnconnectedPing from bytes
    pub fn from_bytes(mut data: Bytes) -> Result<Self, &'static str> {
        if data.len() < 25 {
            // Minimum: 1 + 8 + 16 = 25 bytes
            return Err("Data too short for UnconnectedPing packet");
        }

        // Check packet ID
        let packet_id = data.get_u8();
        if packet_id != UNCONNECTED_PING_ID {
            return Err("Invalid packet ID for UnconnectedPing");
        }

        // Read ping time (8 bytes)
        let mut ping_time = [0u8; 8];
        data.copy_to_slice(&mut ping_time);

        // Read magic (16 bytes)
        let mut magic = [0u8; 16];
        data.copy_to_slice(&mut magic);

        // Read client ID (8 bytes)
        let mut client_id = [0u8; 8];
        data.copy_to_slice(&mut client_id);

        Ok(Self {
            ping_time,
            magic,
            client_id,
        })
    }
}

impl Default for UnconnectedPing {
    fn default() -> Self {
        Self::new([0; 8], [0; 8])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unconnected_ping_from_real_packet() {
        // Test data from a real packet capture
        let test_bytes = [
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x99, 0xa6, 0x00, 0xff, 0xff, 0x00, 0xfe,
            0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
        ];

        let bytes = Bytes::from(test_bytes.to_vec());
        let ping = UnconnectedPing::from_bytes(bytes).expect("Failed to parse packet");

        // Verify packet structure
        assert_eq!(
            ping.ping_time,
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x99, 0xa6]
        );
        assert_eq!(ping.magic, MAGIC);
    }

    #[test]
    fn test_unconnected_ping_round_trip() {
        // Create a ping packet
        let mut ping = UnconnectedPing::new([0; 8], [0; 8]);
        ping.ping_time = [0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x99, 0xa6];

        // Serialize
        let bytes = ping.build();

        // Deserialize
        let parsed_ping =
            UnconnectedPing::from_bytes(bytes).expect("Failed to parse round-trip packet");

        // Verify data matches
        assert_eq!(ping.ping_time, parsed_ping.ping_time);
        assert_eq!(ping.magic, parsed_ping.magic);
    }
}
