#[cfg(test)]
mod tests {
    use tjcl_error_common::{ErrorKind, I2cSub, Kind, SystemSub};
    use tjcl_error_format::ErrorFormat;

    #[test]
    fn test_i2c_roundtrip() {
        let err = ErrorKind::I2C(I2cSub::AddressNack);
        let code = err.to_u16();

        // 0x10 << 8 | 0x03 == 0x1003
        assert_eq!(code, 0x1003);

        // デコードの可逆性検証
        assert_eq!(ErrorKind::from_u16(code), Some(err));
    }

    #[test]
    fn test_system_roundtrip() {
        let err = ErrorKind::System(SystemSub::Timeout);
        let code = err.to_u16();

        // 0x00 << 8 | 0x04 == 0x0004
        assert_eq!(code, 0x0004);
        assert_eq!(ErrorKind::from_u16(code), Some(err));
    }

    #[test]
    fn test_unmapped_code_returns_none() {
        // 未割り当て領域（例: 0xFFFF, 未定義の親ID 0xFF01 等）は安全に None
        assert_eq!(ErrorKind::from_u16(0xFFFF), None);
        assert_eq!(ErrorKind::from_u16(0xFF01), None);
        // 親IDが合っていても、子IDが未定義なら None
        assert_eq!(ErrorKind::from_u16(0x10FF), None);
    }

    #[test]
    fn test_string_format() {
        let err = ErrorKind::I2C(I2cSub::AddressNack);

        // 個別文字列
        assert_eq!(err.kind_str(), "I2C");
        assert_eq!(err.detail_str(), "AddressNack");

        // まとめて文字列
        assert_eq!(err.full_str(), "I2C::AddressNack");
    }
}
