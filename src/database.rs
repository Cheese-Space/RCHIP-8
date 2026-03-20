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
pub const ROMS: &str = include_str!("../assets/programs.json");
pub const SHA1_HASHES: &str = include_str!("../assets/sha1-hashes.json");
#[derive(Default)]
pub enum Compatibility {
    Compatible,
    NotCompatible,
    #[default]
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
#[derive(Default)]
pub struct RomInfo {
    pub title: String,
    pub compatibility: Compatibility
}