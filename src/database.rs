use sha1_smol as sha1;
use std::collections::HashMap;
use std::fmt;
/*
files taken from: https://github.com/chip-8/chip-8-database
license:
Copyright 2023 The CHIP-8 database authors
Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated
documentation files (the “Software”), to deal in the Software without restriction, including without limitation
the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit
persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the
Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE
AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/
const ROMS: &str = include_str!("../assets/programs.json");
const SHA1_HASHES: &str = include_str!("../assets/sha1-hashes.json");
pub enum Compatibility {
    Compatible,
    NotCompatible,
    NotInList
}
impl fmt::Display for Compatibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInList => write!(f, "warning: program is not in rom database so compatibility is not guaranteed"),
            Self::NotCompatible => write!(f, "warning: program is not compatible with the original chip-8\nthe program may run, but not correctly"),
            _ => unreachable!("'compatible' should never be displayed")
        }
    }
}
pub fn check_compatability(program: &[u8]) -> Compatibility {
    let hash = sha1::Sha1::from(program).digest().to_string();
    let sha1_hashes: HashMap<String, usize> = serde_json::from_str(SHA1_HASHES).expect("sha1-hashes.json should be correct json");
    let rom_index = match sha1_hashes.get(&hash) {
        Some(i) => *i,
        None => return Compatibility::NotInList,
    };
    let roms: serde_json::Value = serde_json::from_str(ROMS).expect("programs.json should be valid json");
    let platforms = roms[rom_index]["roms"][&hash]["platforms"].as_array().expect("platform list should be a valid array");
    if platforms[0].as_str().expect("platforms should contain an array of strings") != "originalChip8" {
        return Compatibility::NotCompatible;
    }
    Compatibility::Compatible
}