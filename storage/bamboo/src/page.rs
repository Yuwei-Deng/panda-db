use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Cursor};
use bytes::buf::Reader;
use crate::BMB_PAGE_SIZE;
use bytes::Buf;

type PageLsn = u64;
type LocationIndex = u16;
type TransactionId = u32;

/// lp_off:15,		/* offset to tuple (from start of page) */
/// lp_flags:2,		/* state of line pointer, see below */
/// lp_len:15;		/* byte length of tuple */
struct ItemIdData(u32);


const LP_OFF_SET:u32 = 0b111111111111111;

impl ItemIdData {
    pub fn lp_off(&self)->u16 {
        self>>16
    }
}


///
/// BufferPage
///
struct BufferPage<'a, T> where T:AsRef<[u8]>{
    pd_lsn:PageLsn,
    pd_checksum:u16,
    pd_flags:u16,
    pd_lower:LocationIndex, /* offset to start of free space */
    pd_upper:LocationIndex, /* offset to end of free space */
    pd_special:LocationIndex, /* offset to start of special space */
    pd_pagesize_version:u16,
    pd_prune_xid:TransactionId,
    idx:&'a[u16],
    datum: &'a[T],
}

struct RawPage<'a>(&'a[u8]);

impl<'a> RawPage<'a> {
    pub fn from_mmap()->RawPage<'a>{

    }
}

impl<'a, T> BufferPage<'a, T>{
    pub fn new()-> BufferPage<'a, T>{
        BufferPage{
            idx: &[],
            datum: &[]
        }
    }

    pub fn from_buffer(buffer:&mut [u8])->BufferPage<'a, T>{
        BufferPage{
            idx: &[],
            datum: &[]
        }
    }
}

///
/// Buffer Iterator
///
struct BufferIterator {
    idx:u16,
}


impl<'a, T> IntoIterator for BufferPage<'a, T>{
    type Item = T;
    type IntoIter = BufferInter;

    fn into_iter(self) -> Self::IntoIter {
        BufferIterator{
            idx: 0,
        }
    }
}
