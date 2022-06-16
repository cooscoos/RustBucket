use std::error;
use std::fs::File;
use std::io::{self, Read};
use chrono::{self, Datelike, Timelike};
use regex::Regex;

use super::memory::Memory; // super is the parent

// Read cpu temp
pub fn read_temp() -> Result<i32, Box<dyn error::Error>> {
    let file = "/sys/class/thermal/thermal_zone0/temp";
    let buffer = read_file(file)?;

    let result = buffer.trim().parse::<u32>()?;

    Ok({ result / 1000 } as i32) // return temperature in Celcius
}

pub fn read_memory() -> Result<Memory, Box<dyn error::Error>> {
    let file = "/proc/meminfo";
    let buffer = read_file(file)?;

    let re =
        Regex::new(r#"(?m)^(?:Buffers|Cached|Mem(?:Total|Free|Available)):.*?(?P<number>\d+)"#)?;

    let mut moo = Vec::new();
    for item in re.captures_iter(&buffer) {
        let m = &item["number"].trim().parse::<u32>()?;
        moo.push(*m);
    }

    Ok(Memory::default(moo[0], moo[1], moo[2], moo[3], moo[4]))
}

fn read_file(file: &str) -> Result<String, io::Error> {
    let mut f = File::open(file)?;

    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    Ok(buffer)
}


pub fn get_time_string() -> String {
    let dt = chrono::offset::Local::now();
    let time = format!(
        "{}-{:0>2}-{:0>2}T{:0>2}:{:0>2}:{:0>2}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    );

    time
}