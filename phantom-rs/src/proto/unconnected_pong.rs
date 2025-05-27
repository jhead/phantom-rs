use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct PongData {
    pub edition: String,
    pub motd: String,
    pub protocol_version: String,
    pub version: String,
    pub players: String,
    pub max_players: String,
    pub server_id: String,
    pub sub_motd: String,
    pub game_mode: String,
    pub game_mode_numeric: String,
    pub port4: String,
    pub port6: String,
}

impl Default for PongData {
    fn default() -> Self {
        Self {
            edition: "MCPE".to_string(),
            motd: "phantom §cServer offline".to_string(),
            protocol_version: "800".to_string(),
            version: "1.31.83".to_string(),
            players: "0".to_string(),
            max_players: "1".to_string(),
            server_id: "13253860892328930865".to_string(),
            sub_motd: "Server Offline".to_string(),
            game_mode: "Creative".to_string(),
            game_mode_numeric: "1".to_string(),
            port4: "19132".to_string(),
            port6: "19132".to_string(),
        }
    }
}

impl PongData {
    /// Creates a PongData from a semicolon-separated string
    pub fn from_string(data: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = data.split(';').collect();

        // We need at least 10 fields, but can handle more or fewer gracefully
        if parts.is_empty() {
            return Err("Empty pong data string");
        }

        let mut pong = Self::default();

        // Map fields in order, using default values if not present
        if parts.len() > 0 {
            pong.edition = parts[0].to_string();
        }
        if parts.len() > 1 {
            pong.motd = parts[1].to_string();
        }
        if parts.len() > 2 {
            pong.protocol_version = parts[2].to_string();
        }
        if parts.len() > 3 {
            pong.version = parts[3].to_string();
        }
        if parts.len() > 4 {
            pong.players = parts[4].to_string();
        }
        if parts.len() > 5 {
            pong.max_players = parts[5].to_string();
        }
        if parts.len() > 6 {
            pong.server_id = parts[6].to_string();
        }
        if parts.len() > 7 {
            pong.sub_motd = parts[7].to_string();
        }
        if parts.len() > 8 {
            pong.game_mode = parts[8].to_string();
        }
        if parts.len() > 9 {
            pong.game_mode_numeric = parts[9].to_string();
        }
        if parts.len() > 10 {
            pong.port4 = parts[10].to_string();
        }
        if parts.len() > 11 {
            pong.port6 = parts[11].to_string();
        }

        Ok(pong)
    }
}

impl Into<String> for PongData {
    fn into(self) -> String {
        let fields = vec![
            self.edition.as_str(),
            self.motd.as_str(),
            self.protocol_version.as_str(),
            self.version.as_str(),
            self.players.as_str(),
            self.max_players.as_str(),
            self.server_id.as_str(),
            self.sub_motd.as_str(),
            self.game_mode.as_str(),
            self.game_mode_numeric.as_str(),
            self.port4.as_str(),
            self.port6.as_str(),
        ];

        let joined = fields.join(";");
        format!("{};", joined)
    }
}

// Packet constants
pub const UNCONNECTED_PONG_ID: u8 = 0x1c;

// Magic bytes used in the protocol
pub const MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

#[derive(Debug, Clone)]
pub struct UnconnectedPong {
    pub ping_time: [u8; 8],
    pub server_guid: [u8; 8],
    pub magic: [u8; 16],
    pub pong: PongData,
}

impl UnconnectedPong {
    /// Creates a new UnconnectedPong with default values
    pub fn new() -> Self {
        Self {
            ping_time: [0; 8],
            server_guid: [0; 8],
            magic: MAGIC,
            pong: PongData::default(),
        }
    }

    /// Serializes the UnconnectedPong into bytes for the 0x1c packet
    pub fn build(&self) -> Bytes {
        let mut buf = BytesMut::new();

        // Packet ID
        buf.put_u8(UNCONNECTED_PONG_ID);

        // Ping time (8 bytes)
        buf.put_slice(&self.ping_time);

        // ID (8 bytes)
        buf.put_slice(&self.server_guid);

        // Magic (16 bytes)
        buf.put_slice(&self.magic);

        // Pong data
        let pong_string: String = self.pong.clone().into();
        let pong_bytes = pong_string.as_bytes();

        // Pong data length (2 bytes, big endian)
        buf.put_u16(pong_bytes.len() as u16);

        // Pong data
        buf.put_slice(pong_bytes);

        buf.freeze()
    }

    /// Deserializes an UnconnectedPong from bytes
    pub fn from_bytes(mut data: Bytes) -> Result<Self, &'static str> {
        if data.len() < 35 {
            // Minimum: 1 + 8 + 8 + 16 + 2 = 35 bytes
            return Err("Data too short for UnconnectedPong packet");
        }

        // Check packet ID
        let packet_id = data.get_u8();
        if packet_id != UNCONNECTED_PONG_ID {
            return Err("Invalid packet ID for UnconnectedPong");
        }

        // Read ping time (8 bytes)
        let mut ping_time = [0u8; 8];
        data.copy_to_slice(&mut ping_time);

        // Read ID (8 bytes)
        let mut server_guid = [0u8; 8];
        data.copy_to_slice(&mut server_guid);

        // Read magic (16 bytes)
        let mut magic = [0u8; 16];
        data.copy_to_slice(&mut magic);

        // Read pong data length
        if data.remaining() < 2 {
            return Err("Not enough data for pong length");
        }
        let pong_len = data.get_u16() as usize;

        // Read pong data
        if data.remaining() < pong_len {
            return Err("Not enough data for pong content");
        }
        let pong_bytes = data.split_to(pong_len);
        let pong_string =
            String::from_utf8(pong_bytes.to_vec()).map_err(|_| "Invalid UTF-8 in pong data")?;

        let pong = PongData::from_string(&pong_string)?;

        Ok(Self {
            ping_time,
            server_guid,
            magic,
            pong,
        })
    }
}

impl Default for UnconnectedPong {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unconnected_ping_from_real_packet() {
        // Test data from a real packet capture
        let test_bytes = [
            0x1c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x99, 0xa6, 0xa2, 0x09, 0x63, 0x85, 0x9f,
            0xd0, 0x03, 0xd7, 0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd,
            0xfd, 0x12, 0x34, 0x56, 0x78, 0x00, 0x63, 0x4d, 0x43, 0x50, 0x45, 0x3b, 0x44, 0x65,
            0x64, 0x69, 0x63, 0x61, 0x74, 0x65, 0x64, 0x20, 0x53, 0x65, 0x72, 0x76, 0x65, 0x72,
            0x3b, 0x38, 0x30, 0x30, 0x3b, 0x31, 0x2e, 0x32, 0x31, 0x2e, 0x38, 0x33, 0x3b, 0x30,
            0x3b, 0x31, 0x30, 0x3b, 0x31, 0x31, 0x36, 0x37, 0x35, 0x39, 0x37, 0x32, 0x39, 0x33,
            0x34, 0x34, 0x39, 0x37, 0x37, 0x33, 0x31, 0x35, 0x34, 0x33, 0x3b, 0x42, 0x65, 0x64,
            0x72, 0x6f, 0x63, 0x6b, 0x20, 0x6c, 0x65, 0x76, 0x65, 0x6c, 0x3b, 0x53, 0x75, 0x72,
            0x76, 0x69, 0x76, 0x61, 0x6c, 0x3b, 0x31, 0x3b, 0x31, 0x39, 0x31, 0x33, 0x32, 0x3b,
            0x31, 0x39, 0x31, 0x33, 0x33, 0x3b, 0x30, 0x3b,
        ];

        let bytes = Bytes::from(test_bytes.to_vec());
        let ping = UnconnectedPong::from_bytes(bytes).expect("Failed to parse packet");

        // Verify packet structure
        assert_eq!(
            ping.ping_time,
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x99, 0xa6]
        );
        assert_eq!(
            ping.server_guid,
            [0xa2, 0x09, 0x63, 0x85, 0x9f, 0xd0, 0x03, 0xd7]
        );
        assert_eq!(ping.magic, MAGIC);

        // Verify pong data
        assert_eq!(ping.pong.edition, "MCPE");
        assert_eq!(ping.pong.motd, "Dedicated Server");
        assert_eq!(ping.pong.protocol_version, "800");
        assert_eq!(ping.pong.version, "1.21.83");
        assert_eq!(ping.pong.players, "0");
        assert_eq!(ping.pong.max_players, "10");
        assert_eq!(ping.pong.server_id, "11675972934497731543");
        assert_eq!(ping.pong.sub_motd, "Bedrock level");
        assert_eq!(ping.pong.game_mode, "Survival");
        assert_eq!(ping.pong.game_mode_numeric, "1");
        assert_eq!(ping.pong.port4, "19132");
        assert_eq!(ping.pong.port6, "19133");
    }

    #[test]
    fn test_unconnected_ping_round_trip() {
        // Create a ping packet
        let mut ping = UnconnectedPong::new();
        ping.ping_time = [0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x99, 0xa6];
        ping.server_guid = [0xa2, 0x09, 0x63, 0x85, 0x9f, 0xd0, 0x03, 0xd7];

        ping.pong.edition = "MCPE".to_string();
        ping.pong.motd = "Test Server".to_string();
        ping.pong.protocol_version = "800".to_string();
        ping.pong.version = "1.21.83".to_string();
        ping.pong.players = "5".to_string();
        ping.pong.max_players = "20".to_string();

        // Serialize
        let bytes = ping.build();

        // Deserialize
        let parsed_ping =
            UnconnectedPong::from_bytes(bytes).expect("Failed to parse round-trip packet");

        // Verify data matches
        assert_eq!(ping.ping_time, parsed_ping.ping_time);
        assert_eq!(ping.server_guid, parsed_ping.server_guid);
        assert_eq!(ping.magic, parsed_ping.magic);
        assert_eq!(ping.pong.edition, parsed_ping.pong.edition);
        assert_eq!(ping.pong.motd, parsed_ping.pong.motd);
        assert_eq!(
            ping.pong.protocol_version,
            parsed_ping.pong.protocol_version
        );
        assert_eq!(ping.pong.version, parsed_ping.pong.version);
        assert_eq!(ping.pong.players, parsed_ping.pong.players);
        assert_eq!(ping.pong.max_players, parsed_ping.pong.max_players);
    }

    #[test]
    fn test_pong_data_from_string() {
        let pong_string = "MCPE;Dedicated Server;800;1.21.83;0;10;11675972934497731543;Bedrock level;Survival;1;19132;19133;0;";
        let pong = PongData::from_string(pong_string).expect("Failed to parse pong data");

        assert_eq!(pong.edition, "MCPE");
        assert_eq!(pong.motd, "Dedicated Server");
        assert_eq!(pong.protocol_version, "800");
        assert_eq!(pong.version, "1.21.83");
        assert_eq!(pong.players, "0");
        assert_eq!(pong.max_players, "10");
        assert_eq!(pong.server_id, "11675972934497731543");
        assert_eq!(pong.sub_motd, "Bedrock level");
        assert_eq!(pong.game_mode, "Survival");
        assert_eq!(pong.game_mode_numeric, "1");
        assert_eq!(pong.port4, "19132");
        assert_eq!(pong.port6, "19133");
    }

    #[test]
    fn test_pong_data_to_string() {
        let pong = PongData {
            edition: "MCPE".to_string(),
            motd: "Test Server".to_string(),
            protocol_version: "800".to_string(),
            version: "1.21.83".to_string(),
            players: "5".to_string(),
            max_players: "20".to_string(),
            server_id: "123456789".to_string(),
            sub_motd: "Sub MOTD".to_string(),
            game_mode: "Creative".to_string(),
            game_mode_numeric: "1".to_string(),
            port4: "19132".to_string(),
            port6: "19133".to_string(),
        };

        let pong_string: String = pong.into();
        let expected =
            "MCPE;Test Server;800;1.21.83;5;20;123456789;Sub MOTD;Creative;1;19132;19133;";

        assert_eq!(pong_string, expected);
    }
}
