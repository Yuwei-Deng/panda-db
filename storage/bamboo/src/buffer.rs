use memmap2::MmapMut;
use memmap2::MmapOptions;
use std::io::Write;
use std::fs::File;
/// @Copy Right
/// Author Deng Yuwei
/// # buffer Module
///
#[repr(C, packed)]
pub struct User {
    id: u8,
    username: [u8; 20],
}

pub struct RawUser {
    buf: [u8; 21],
}

impl RawUser {
    pub fn as_bytes_mut(&mut self) -> &mut [u8; 21] {
        &mut self.buf
    }

    pub fn as_user_mut(&mut self) -> &mut User {
        unsafe { &mut *(self.buf.as_mut_ptr() as *mut _) }
    }
}

pub fn open_file(data_file:String){
    let file = File::open(data_file).unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    mmap.
    assert_eq!(b"# memmap2", &mmap[0..9]);
}


#[test]
fn test_buffer_ref(){
    let mut raw_user =  RawUser{
        buf: [0;21],
    };

    let user =  raw_user.as_user_mut();
    println!("Userid={:?}, name={:?}", user.id, user.username);
    println!("raw user={:?}", raw_user.buf);
    raw_user.buf[0]=10;
    raw_user.buf[10]=10;
    let user =  raw_user.as_user_mut();
    println!("Userid={:?}, name={:?}", user.id, user.username);
}

#[test]
fn test_mmap(){
    use memmap2::MmapOptions;
    use std::io::Write;
    use std::fs::File;
    let file = File::open("/tmp/test.data").unwrap();
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    assert_eq!(b"# memmap2", &mmap[0..9]);
}
