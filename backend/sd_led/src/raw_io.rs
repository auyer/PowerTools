use std::fs::OpenOptions;
use std::io::{Error, Read, Seek, SeekFrom, Write};

#[inline]
pub fn write2(p0: u8, p1: u8) -> Result<usize, Error> {
    write_to(0x6c, 0x81)?;
    wait_ready_for_write()?;
    let count0 = write_to(0x68, p0)?;
    wait_ready_for_write()?;
    let count1 = write_to(0x68, p1)?;
    Ok(count0 + count1)
}

#[inline]
pub fn write_read(p0: u8) -> Result<u8, Error> {
    write_to(0x6c, 0x81)?;
    wait_ready_for_write()?;
    write_to(0x68, p0)?;
    wait_ready_for_read()?;
    read_from(0x68)
}

pub fn write_to(location: u64, value: u8) -> Result<usize, Error> {
    let mut file = OpenOptions::new().write(true).open("/dev/port")?;
    file.seek(SeekFrom::Start(location))?;
    file.write(&[value])
}

pub fn read_from(location: u64) -> Result<u8, Error> {
    let mut file = OpenOptions::new().read(true).open("/dev/port")?;
    file.seek(SeekFrom::Start(location))?;
    let mut buffer = [0];
    file.read(&mut buffer)?;
    Ok(buffer[0])
}

pub fn wait_ready_for_write() -> Result<(), Error> {
    let mut count = 0;
    while count < 0x1ffff && (read_from(0x6c)? & 2) != 0 {
        count += 1;
    }
    Ok(())
}

pub fn wait_ready_for_read() -> Result<(), Error> {
    let mut count = 0;
    while count < 0x1ffff && (read_from(0x6c)? & 1) == 0 {
        count += 1;
    }
    Ok(())
}
