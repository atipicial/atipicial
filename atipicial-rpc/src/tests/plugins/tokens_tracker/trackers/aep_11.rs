use super::{Aep11BalanceKey, Aep11TransferKey};
use atipicial_io::{BinaryWriter, Serializable};
use atipicial_primitives::UInt160;

fn hash(seed: u8) -> UInt160 {
    UInt160::from_bytes(&[seed; 20]).expect("valid UInt160")
}

#[test]
fn aep11_key_size_matches_serialized_varbytes_boundaries() {
    for token_len in [252usize, 253] {
        let token = vec![0xAB; token_len];
        let balance = Aep11BalanceKey::new(hash(1), hash(2), token.clone());
        let transfer = Aep11TransferKey::new(hash(3), 1_716_151_234_567, hash(4), token, 7);

        let mut balance_writer = BinaryWriter::new();
        balance.serialize(&mut balance_writer).unwrap();
        let balance_bytes = balance_writer.into_bytes();
        assert_eq!(balance.size(), balance_bytes.len());

        let mut transfer_writer = BinaryWriter::new();
        transfer.serialize(&mut transfer_writer).unwrap();
        let transfer_bytes = transfer_writer.into_bytes();
        assert_eq!(transfer.size(), transfer_bytes.len());

        if token_len == 252 {
            assert_eq!(balance_bytes[40], 0xFC);
            assert_eq!(transfer_bytes[transfer.base.size()], 0xFC);
        } else {
            assert_eq!(&balance_bytes[40..43], &[0xFD, 0xFD, 0x00]);
            assert_eq!(
                &transfer_bytes[transfer.base.size()..transfer.base.size() + 3],
                &[0xFD, 0xFD, 0x00]
            );
        }
    }
}
