extern crate ubnt_info;
use ubnt_info::board_info::*;

fn main() {

    let eeprom_dev = BoardInfo::get_mtd_part("eeprom").expect("Couldn't find eeprom partition");
    let info = BoardInfo::new(&eeprom_dev);
    println!("INFO: {:?}", info);
}
