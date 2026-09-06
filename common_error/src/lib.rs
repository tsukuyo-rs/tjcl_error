#![no_std]

#[path = "error/macros.rs"]
mod macros;

mod error {
    pub mod i2c;
    pub mod kind;
    pub mod system;
}

pub use error::i2c::I2cSub;
pub use error::kind::{ErrorKind, Kind};
pub use error::system::SystemSub;
