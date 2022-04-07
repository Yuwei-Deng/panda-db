use std::fs::File;
use std::io::Read;

mod btree;
mod page;
mod frame;
mod buffer;

///
/// Constant of storage module
///
const BMB_PAGE_SIZE: u16 = 1024*16;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized `Result` type for mini-redis operations.
///
/// This is defined as a convenience.
pub type Result<T> = std::result::Result<T, Error>;

pub const SAMPLE_MARK:u8 = b'-';

/// CRDT data between nodes
pub const REPLICATE_MARK:u8= b'#';

/// gossip command data
pub const COMMAND_MARK:u8 = b'+';

pub const VARIANT_LENGTH_MARK:u8 = b'@';

/// setting of BUF
pub const READ_BUF_SIZE: usize = 4*1024;




#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
