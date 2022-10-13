pub use ws_messages_macros::*;

#[cfg(test)]
mod tests {
    use crate as ws_messages;
    use ws_bitpack::*;
    use ws_messages::*;
    use ws_messages::reader::*;
    use ws_messages::writer::*;

    use std::io::{Cursor, Read};

    #[derive(MessageStruct)]
    struct Message0002 {
        build_number: u32,
        realm_id: u32,
        realm_group_id: u32,
        realm_group_enum: u32,
        startup_time: u64,
        listen_port: u16,
        #[packed(5)]
        connection_type: u8,
        network_message_crc: u32,
        process_id: u32,
        process_creation_time: u64,
    }

    #[test]
    fn test_simple_read() {
        let data = "2f00000240c00000000000008800000000000000000000\
            00000000000000489208b89c000000000000000000000000";
        let data = hex::decode(data).unwrap();
        let mut cursor = Cursor::new(&data);
        let mut reader = BitPackReader::new(&mut cursor);

        // header
        reader.read_u64(24).unwrap();
        reader.read_u64(11).unwrap();

        let result: Message0002 = MessageReader::read(&mut reader).unwrap();
        assert_eq!(result.build_number, 6152);
        assert_eq!(result.realm_id, 0);
        assert_eq!(result.realm_group_id, 17);
        assert_eq!(result.realm_group_enum, 0);
        assert_eq!(result.startup_time, 0);
        assert_eq!(result.listen_port, 0);
        assert_eq!(result.connection_type, 9);
        assert_eq!(result.network_message_crc, 2629306514);
        assert_eq!(result.process_id, 0);
        assert_eq!(result.process_creation_time, 0);
    }

    #[test]
    fn test_simple_write() {
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = BitPackWriter::new(&mut cursor);

        // header
        assert!(writer.write_u64(47, 24).is_ok());
        assert!(writer.write_u64(2, 11).is_ok());

        let message = Message0002 {
            build_number: 6152,
            realm_id: 0,
            realm_group_id: 17,
            realm_group_enum: 0,
            startup_time: 0,
            listen_port: 0,
            connection_type: 9,
            network_message_crc: 2629306514,
            process_id: 0,
            process_creation_time: 0,
        };

        MessageWriter::write(&mut writer, &message).unwrap();

        // flush data
        assert!(writer.align_and_flush().is_ok());

        // check final buffer
        cursor.set_position(0);
        let mut vec = Vec::new();
        cursor.read_to_end(&mut vec).unwrap();
        assert_eq!(
            hex::encode(&vec),
            "2f00000240c00000000000008800000000000000000000\
            00000000000000489208b89c000000000000000000000000"
        );
    }
}
