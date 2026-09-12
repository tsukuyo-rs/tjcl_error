use crate::define_error_kind;
use crate::error::i2c::I2cSub;
use crate::error::system::SystemSub;

define_error_kind! {
    /// システム全般の異常
    System = 0x00 => SystemSub,
    /// I2Cペリフェラル通信異常
    I2C = 0x10 => I2cSub,
}
