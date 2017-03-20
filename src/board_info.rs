use std::io::prelude::*;
use std::fs::File;
use std::str;
use std::path::Path;
use std::env;

use std::io::BufReader;
use std::io::BufRead;

extern crate regex;
use self::regex::Regex;

// http://stackoverflow.com/questions/25410028/how-to-read-a-struct-from-a-file-in-rust
// https://www.reddit.com/r/rust/comments/3rrqal/interpreting_binary_data_into_other_types/

#[derive(Debug)]
pub struct BoardInfo {
    mac: [u8; 6],
    mrev: u16,
    serial: String,
    board_name: String,
    board_code: String,
    kidx: u8
}

impl BoardInfo {
    pub fn new(path: &str) -> BoardInfo {
        #[repr(C)]
        struct BoardInfoC {
            header: [u8; 32],
            crc16: u16,
            mac: [u8; 6],
            mrev: u16,
            serial: [u8; 32],
            board_type: [u8; 32],
            padding0: [u8; 54],
            kidx: u8,
            padding1: [u8; 1]
        }

        let mut file = File::open(path).unwrap();

        let mut buffer = [0; 162];
        file.read_exact(&mut buffer).unwrap();

        let data_ptr: *const u8 = buffer.as_ptr();
        let header_ptr: *const BoardInfoC = data_ptr as *const _;
        let header_ref: &BoardInfoC = unsafe { &*header_ptr };

        let board_code = BoardInfo::get_string(&header_ref.board_type);
        let board_name = BoardInfo::get_board_desc(&board_code);

        BoardInfo {
            mac: header_ref.mac,
            mrev: header_ref.mrev,
            serial: BoardInfo::get_string(&header_ref.serial),
            board_name: board_name,
            board_code: board_code,
            kidx: header_ref.kidx
        }
    }

    pub fn get_mtd_part(partname: &str) -> Result<String, String> {
        // https://doc.rust-lang.org/regex/regex/index.html
        lazy_static! {
            static ref RE_MTDPART: Regex  = Regex::new(r##"^(.+): ([0-9a-f]+) ([0-9a-f]+) "(.+)"$"##).unwrap();
        }

        // For testing
        let proc_root = match env::var("PROC_ROOT") {
            Ok(val) => val,
            Err(_error) => "/proc".to_string()
        };

        let dev_root = match env::var("DEV_ROOT") {
            Ok(val) => val,
            Err(_error) => "/dev".to_string()
        };

        let mtd_info = Path::new(&proc_root).join("mtd");
        let mtd_fd = File::open(mtd_info).expect("Unable to open /proc/mtd");
        let file = BufReader::new(&mtd_fd);

        for (_line_num, result) in file.lines().enumerate() {
            let line = result.unwrap();
            let line_str = &line;

            if RE_MTDPART.is_match(line_str) {
                let captures = RE_MTDPART.captures(line_str).unwrap();
                let part_dev = &captures[1];
                let part_name = &captures[4];
                if part_name == partname {
                    let path = Path::new(&dev_root).join(part_dev)
                        .into_os_string()
                        .into_string()
                        .expect("Couldn't make PathBuf into a string");
                    return Ok(path.to_string());
                }
            }
        }
        return Err("Partition not found".to_string());
    }

    fn get_board_desc(board_type: &str) -> String {
        (match &*board_type {
            "e50" => "EdgeRouter X",
            "e51" => "EdgeRouter X SFP",
            _     => "Unknown board"
        }).to_string()
    }

    // Gross
    fn get_string(raw: &[u8]) -> String {
        let raw_str = str::from_utf8(&raw).unwrap();
        let re = Regex::new("(\u{0})+").unwrap();
        re.replace_all(raw_str, "").to_string()
    }
}
