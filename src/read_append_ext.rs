use std::{io::Read, mem::MaybeUninit};

unsafe fn as_initialized(slice: &mut [MaybeUninit<u8>]) -> &mut [u8] {
    let len = slice.len();
    let ptr = slice.as_mut_ptr() as *mut u8;
    unsafe { std::slice::from_raw_parts_mut(ptr, len) }
}

pub trait ReadAppendExt {
    fn read_append(&mut self, buf: &mut Vec<u8>, max_read_size: usize) -> std::io::Result<usize>;
}

impl<T> ReadAppendExt for T
where
    T: Read,
{
    fn read_append(&mut self, buf: &mut Vec<u8>, max_read_size: usize) -> std::io::Result<usize> {
        buf.reserve(max_read_size);
        let old_length = buf.len();
        let spare_cap = unsafe { as_initialized(buf.spare_capacity_mut()) };
        let count = self.read(spare_cap)?;
        unsafe {
            buf.set_len(old_length + count);
        }
        Ok(count)
    }
}
