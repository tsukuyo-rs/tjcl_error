use tjcl_error_common::{ErrorKind, I2cSub, Kind, SystemSub};

/// エラー文字列化トレイト（純粋な core プリミティブ &'static str のみ返却）
pub trait ErrorFormat {
    /// ① Kind部分の文字列を取得
    fn kind_str(&self) -> &'static str;

    /// ① Detail部分の文字列を取得
    fn detail_str(&self) -> &'static str;

    /// ② まとめて文字列化した完全名を取得（例: "I2C::AddressNack"）
    fn full_str(&self) -> &'static str;
}

impl ErrorFormat for ErrorKind {
    fn kind_str(&self) -> &'static str {
        match self.kind() {
            Kind::System => "System",
            Kind::I2C => "I2C",
        }
    }

    fn detail_str(&self) -> &'static str {
        match self {
            ErrorKind::System(sub) => match sub {
                SystemSub::NotInitialized => "NotInitialized",
                SystemSub::AlreadyInitialized => "AlreadyInitialized",
                SystemSub::InvalidParameter => "InvalidParameter",
                SystemSub::Timeout => "Timeout",
                SystemSub::HardwareFault => "HardwareFault",
            },
            ErrorKind::I2C(sub) => match sub {
                I2cSub::BusBusy => "BusBusy",
                I2cSub::ArbitrationLost => "ArbitrationLost",
                I2cSub::AddressNack => "AddressNack",
                I2cSub::DataNack => "DataNack",
                I2cSub::Timeout => "Timeout",
                I2cSub::Overrun => "Overrun",
            },
        }
    }

    fn full_str(&self) -> &'static str {
        match self {
            ErrorKind::System(sub) => match sub {
                SystemSub::NotInitialized => "System::NotInitialized",
                SystemSub::AlreadyInitialized => "System::AlreadyInitialized",
                SystemSub::InvalidParameter => "System::InvalidParameter",
                SystemSub::Timeout => "System::Timeout",
                SystemSub::HardwareFault => "System::HardwareFault",
            },
            ErrorKind::I2C(sub) => match sub {
                I2cSub::BusBusy => "I2C::BusBusy",
                I2cSub::ArbitrationLost => "I2C::ArbitrationLost",
                I2cSub::AddressNack => "I2C::AddressNack",
                I2cSub::DataNack => "I2C::DataNack",
                I2cSub::Timeout => "I2C::Timeout",
                I2cSub::Overrun => "I2C::Overrun",
            },
        }
    }
}
