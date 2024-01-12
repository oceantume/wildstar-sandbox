pub use ws_messages_macros::*;

#[cfg(test)]
mod tests {
    use ws_bitpack::*;
    use crate::*;

    fn write_and_read<T>(input: &T) -> T
    where
        T: WriteValue,
        T: ReadValue,
    {
        let mut buf = [0u8; 65536];
        let mut writer = BitPackWriter::new(&mut buf);
        writer.write(input).unwrap();
        writer.align().unwrap();
        let mut reader = BitPackReader::new(&buf);
        reader.read().unwrap()
    }

    #[test]
    fn test_vec_write_read() {
        #[derive(MessageStruct)]
        struct Struct {
            count: u32,
            #[length(count)]
            items: Vec<u32>,
        }
        let in_value = Struct {
            count: 5,
            items: vec![1, 2, 3, 4, 5],
        };
        let out_value = write_and_read(&in_value);
        assert_eq!(in_value.count, out_value.count);
        assert_eq!(in_value.items, out_value.items);
    }

    #[test]
    fn test_packed_write_read() {
        #[derive(MessageStruct)]
        struct Struct {
            #[packed(5)]
            value: u32,
        }
        let in_value = Struct { value: 5 };
        let out_value = write_and_read(&in_value);
        assert_eq!(in_value.value, out_value.value);
    }

    #[test]
    fn test_packed_vec_write_read() {
        #[derive(MessageStruct)]
        struct Struct {
            count: u32,
            #[packed(5)]
            #[length(count)]
            items: Vec<u32>,
        }
        let in_value = Struct {
            count: 5,
            items: vec![1, 2, 3, 4, 5],
        };
        let out_value = write_and_read(&in_value);
        assert_eq!(in_value.count, out_value.count);
        assert_eq!(in_value.items, out_value.items);
    }

    #[test]
    #[should_panic(expected = "Invalid union variant 2")]
    fn test_union() {
        #[derive(MessageUnion)]
        enum Union {
            Unsigned64 { value: u64 },
            Signed16 { value: i16 },
        }
        #[derive(MessageStruct)]
        struct Struct {
            id: u32,
            #[variant(id)]
            union: Union,
        }

        // test first variant
        let in_value = Struct {
            id: 0,
            union: Union::Unsigned64 {
                value: 123456789123456789,
            },
        };
        let out_value = write_and_read(&in_value);
        let out_union_value = match out_value.union {
            Union::Unsigned64 { value } => Some(value),
            _ => None,
        };
        assert_eq!(out_union_value, Some(123456789123456789));
        assert_eq!(out_union_value.unwrap().bits(), 64);

        // test second variant
        let in_value = Struct {
            id: 1,
            union: Union::Signed16 { value: -12349 },
        };
        let out_value = write_and_read(&in_value);
        let out_union_value = match out_value.union {
            Union::Signed16 { value } => Some(value),
            _ => None,
        };
        assert_eq!(out_union_value, Some(-12349));
        assert_eq!(out_union_value.unwrap().bits(), 16);

        // test invalid variant (should panic during read with above message)
        let in_value = Struct {
            id: 2,
            union: Union::Signed16 { value: 0 },
        };
        write_and_read(&in_value);
    }

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
        let mut reader = BitPackReader::new(&data);

        // header
        reader.read_u64(24).unwrap();
        reader.read_u64(11).unwrap();

        let result: Message0002 = reader.read().unwrap();
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
        let mut buf = [0u8; 47];
        let mut writer = BitPackWriter::new(&mut buf);

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

        // header
        assert!(writer.write_u64(47, 24).is_ok());
        assert!(writer.write_u64(2, 11).is_ok());
        writer.write(&message).unwrap();

        // check final buffer
        assert_eq!(
            hex::encode(&buf),
            "2f00000240c00000000000008800000000000000000000\
            00000000000000489208b89c000000000000000000000000"
        );
    }

    #[test]
    fn test_simple_bits() {
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

        assert_eq!(message.bits(), 341);
    }

    #[derive(MessageStruct)]
    struct Message02EE {
        account_id: u32,
        #[aligned]
        session_guid: [u8; 16],
        account_name: String,
    }

    #[test]
    fn test_message_2() {
        let data: Vec<u8> = hex::decode(
            "2a0000ee0aae010000ba75a452f8a21b49b0d886ed\
            0d9e58a81063006c0061006d006f0075006e006500",
        )
        .unwrap();

        let guid_data = hex::decode("ba75a452f8a21b49b0d886ed0d9e58a8").unwrap();
        let mut reader = BitPackReader::new(&data);

        // header
        reader.read_u64(24).unwrap();
        reader.read_u64(11).unwrap();

        let result: Message02EE = reader.read().unwrap();
        assert_eq!(result.account_id, 13761);
        assert_eq!(result.session_guid, guid_data.as_slice());
        assert_eq!(result.account_name, "clamoune");
    }
}
