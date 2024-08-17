#![feature(seek_stream_len)]

use std::fs::File;
use std::io::prelude::*;

static MAGIC: [u8; 4] = [66, 80, 83, 49];

fn vle_decode(file: &mut File) -> u64 {
    let mut data: u64 = 0;
    let mut shift: u64 = 1;
    loop {
        let mut buf = [0; 1];
        let _ = file.read(&mut buf);
        let x: u64 = buf[0].into();
        data += (x & 0x7f) * shift;
        if (x & 0x80) != 0 {
            break;
        }
        shift <<= 7;
        data += shift;
    }
    data
}

fn read_sizes(file: &mut File) -> (u64, u64) {
    let source_size = vle_decode(file);
    let target_size = vle_decode(file);
    (source_size, target_size)
}

fn read_metadata(file: &mut File) -> String {
    let metadata_size = vle_decode(file);
    let mut take = file.take(metadata_size);
    let mut buf = vec![];
    let _ = take.read_to_end(&mut buf);
    match std::str::from_utf8(buf.as_slice()) {
        Ok(v) => v.to_string(),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}

fn read_actions(file: &mut File) -> u64 {
    let mut output_offset: u64 = 0;
    let mut source_offset: i64 = 0;
    let mut target_offset: i64 = 0;
    let mut actions: u64 = 0;
    loop {
        let pos = file
            .stream_position()
            .expect("no stream pos? [insert megamind]");
        let len = file.stream_len().expect("no stream len? [insert megamind]");
        if pos >= len - 12 {
            break;
        }

        let data = vle_decode(file);
        let command = data & 3;
        let length = (data >> 2) + 1;

        match command {
            0 => {
                println!(
                    "Copy starting from {} with length {} from source to target.",
                    output_offset, length
                );
                output_offset += length;
            }
            1 => {
                println!(
                    "Copy the following {} bytes from the patch file to the target.",
                    length
                );

                let mut take = file.take(length);
                let mut buf = vec![];
                let _ = take.read_to_end(&mut buf);
                println!("{:?}", buf);

                /*let _ = file.seek_relative(
                    length
                        .try_into()
                        .expect("whoopsies couldnt convert u64 to i64"),
                ); // temporary */
                output_offset += length;
            }
            2 => {
                let data2 = vle_decode(file);
                source_offset += (if data2 & 1 != 0 { -1i64 } else { 1i64 }) * (data2 >> 1) as i64;
                println!(
                    "Copy starting from {} with length {} in source to {} in target.",
                    output_offset, length, source_offset
                );
                output_offset += length;
                source_offset += length as i64;
            }
            3 => {
                let data2 = vle_decode(file);
                target_offset += (if data2 & 1 != 0 { -1i64 } else { 1i64 }) * (data2 >> 1) as i64;
                println!(
                    "Copy starting from {} with length {} in target to {} in target.",
                    output_offset, length, target_offset
                );
                output_offset += length;
                target_offset += length as i64;
            }
            _ => panic!("holy fucking shit explod now!"),
        }

        actions += 1;
    }

    actions
}

fn read_crcs(file: &mut File) -> ([u8; 4], [u8; 4], [u8; 4]) {
    let mut buf = [0; 4];
    let _ = file.read(&mut buf);
    let sc = buf;
    let _ = file.read(&mut buf);
    let tc = buf;
    let _ = file.read(&mut buf);
    let pc = buf;
    (sc, tc, pc)
}

fn main() {
    let mut file = File::open("patch.bps").expect("womp womp file no open");

    let mut buf = [0; 4];
    let _ = file.read(&mut buf);
    assert_eq!(buf, MAGIC);

    let sizes = read_sizes(&mut file);
    let metadata = read_metadata(&mut file);
    let actions = read_actions(&mut file);
    let crcs = read_crcs(&mut file);

    println!("Input File Size:  {}", sizes.0);
    println!("Output File Size: {}", sizes.1);
    println!("Metadata:         {}", metadata);
    println!("Total Actions:    {}", actions);
    println!("Source CRC:       {:02X?}", crcs.0);
    println!("Target CRC:       {:02X?}", crcs.1);
    println!("Patch CRC:        {:02X?}", crcs.2);
}
