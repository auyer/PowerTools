//! Rough Rust port of some BatCtrl functionality
//! Original: /usr/share/jupiter_controller_fw_updater/RA_bootloader_updater/linux_host_tools/BatCtrl
//! I do not have access to the source code, so this is my own interpretation of what it does.
//!
//! But also Quanta is based in a place with some questionable copyright practices, so...
pub mod raw_io;

use std::io::Error;

pub fn set_led(red_unused: bool, green_aka_white: bool, blue_unused: bool) -> Result<usize, Error> {
    let payload: u8 = 0x80
        | (red_unused as u8 & 1)
        | ((green_aka_white as u8 & 1) << 1)
        | ((blue_unused as u8 & 1) << 2);
    //log::info!("Payload: {:b}", payload);
    raw_io::write2(Setting::LEDStatus as _, payload)
}

pub fn set(setting: Setting, mode: u8) -> Result<usize, Error> {
    raw_io::write2(setting as u8, mode)
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Setting {
    CycleCount = 0x32,
    ControlBoard = 0x6C,
    Charge = 0xA6,
    ChargeMode = 0x76,
    LEDStatus = 199,
    LEDBreathing = 0x63,
    FanSpeed = 0x2c, // lower 3 bits seem to not do everything, every other bit increases speed -- 5 total steps, 0xf4 seems to do something similar too
    // 0x40 write 0x08 makes LED red + green turn on
    // 0x58 write 0x80 shuts off battery power (bms?)
    // 0x63 makes blue (0x02) or white (0x01) LED breathing effect
    // 0x7a write 0x01, 0x02, or 0x03 turns off display
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum ControlBoard {
    Enable = 0xAA,
    Disable = 0xAB,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum ChargeMode {
    Normal = 0,
    Discharge = 0x42,
    Idle = 0x45,
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Charge {
    Enable = 0,
    Disable = 4,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(dead_code)]
    fn led_all_experiment_test() -> Result<(), Error> {
        let original = raw_io::write_read(Setting::LEDStatus as _)?;
        let sleep_dur = std::time::Duration::from_millis(1000);
        for b in 0..0x7F {
            let actual = 0x80 | b;
            raw_io::write2(Setting::LEDStatus as _, actual)?;
            println!("Wrote {actual:#b} to LED byte");
            std::thread::sleep(sleep_dur);
        }
        raw_io::write2(Setting::LEDStatus as _, original)?;
        Ok(())
    }

    #[test]
    #[allow(dead_code)]
    fn led_singles_experiment_test() -> Result<(), Error> {
        let original = raw_io::write_read(Setting::LEDStatus as _)?;
        let sleep_dur = std::time::Duration::from_millis(1000);
        let mut value = 1;
        for _ in 0..std::mem::size_of::<u8>()*8 {
            let actual = 0x80 | value;
            raw_io::write2(Setting::LEDStatus as _, actual)?;
            println!("Wrote {actual:#b} to LED byte");
            value = value << 1;
            std::thread::sleep(sleep_dur);
        }
        raw_io::write2(Setting::LEDStatus as _, original)?;
        Ok(())
    }

    #[test]
    #[allow(dead_code)]
    fn led_specify_experiment_test() -> Result<(), Error> {
        let mut buffer = String::new();
        println!("LED number(s) to display?");
        std::io::stdin().read_line(&mut buffer)?;

        let mut resultant = 0;
        let original = raw_io::write_read(Setting::LEDStatus as _)?;
        for word in buffer.split(' ') {
            let trimmed_word = word.trim();
            if !trimmed_word.is_empty() {
                let value: u8 = trimmed_word.parse().expect("Invalid u8 number");
                let actual = 0x80 | value;
                raw_io::wait_ready_for_write()?;
                raw_io::write2(Setting::LEDStatus as _, actual)?;
                println!("Wrote {actual:#b} to LED byte");
                resultant |= actual;
            }
        }
        println!("Effectively wrote {resultant:#b} to LED byte");

        println!("Press enter to return to normal");
        std::io::stdin().read_line(&mut buffer)?;
        raw_io::write2(Setting::LEDStatus as _, original)?;
        Ok(())
    }

    #[test]
    #[allow(dead_code)]
    fn breath_specify_experiment_test() -> Result<(), Error> {
        let mut buffer = String::new();
        println!("LED number(s) to display?");
        std::io::stdin().read_line(&mut buffer)?;

        for word in buffer.split(' ') {
            let trimmed_word = word.trim();
            if !trimmed_word.is_empty() {
                let value: u8 = trimmed_word.parse().expect("Invalid u8 number");
                let actual = 0x20 | value;
                raw_io::wait_ready_for_write()?;
                raw_io::write2(0x63, actual)?;
                println!("Wrote {actual:#b} to LED breathing byte");
            }
        }

        println!("Press enter to return to normal");
        std::io::stdin().read_line(&mut buffer)?;
        raw_io::write2(0x63, 0)?;
        Ok(())
    }

    #[test]
    #[allow(dead_code)]
    fn unmapped_ports_experiment_test() -> Result<(), Error> {
        let sleep_dur = std::time::Duration::from_millis(10000);
        let value = 0xaa;
        for addr in 0x63..0x64 {
            //raw_io::wait_ready_for_read()?;
            //let read = raw_io::write_read(addr)?;
            raw_io::wait_ready_for_write()?;
            raw_io::write2(addr, value)?;
            println!("wrote {value:#b} for {addr:#x} port");
            std::thread::sleep(sleep_dur);
        }
        //raw_io::write2(Setting::LEDStatus as _, 0)?;
        Ok(())
    }

    #[test]
    #[allow(dead_code)]
    fn write_specify_experiment_test() -> Result<(), Error> {
        let mut buffer = String::new();
        println!("Register?");
        std::io::stdin().read_line(&mut buffer)?;
        let register: u8 = buffer.trim().parse().expect("Invalid u8 number");
        buffer.clear();

        println!("Value(s)?");
        std::io::stdin().read_line(&mut buffer)?;

        for word in buffer.split(' ') {
            let trimmed_word = word.trim();
            if !trimmed_word.is_empty() {
                let value: u8 = trimmed_word.parse().expect("Invalid u8 number");
                raw_io::wait_ready_for_write()?;
                raw_io::write2(register, value)?;
                println!("Wrote {value:#09b} to {register:#02x} register");
            }
        }

        println!("Press enter to clear register");
        std::io::stdin().read_line(&mut buffer)?;
        raw_io::write2(register, 0)?;
        Ok(())
    }
}
