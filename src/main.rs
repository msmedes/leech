mod bencode;

use std::fs;
use std::io::Read;

use serde_bencode::de;
extern crate serde_derive;

fn main() {
    // let stdin = io::stdin();
    // let mut buffer = Vec::new();
    // let mut handle = stdin.lock();
    // match handle.read_to_end(&mut buffer) {
    //     Ok(_) => match de::from_bytes::<Torrent>(&buffer) {
    //         Ok(t) => render_torrent(&t),
    //         Err(e) => println!("ERROR: {:?}", e),
    //     },
    //     Err(e) => println!("ERROR: {:?}", e),
    // }
    let filename = "debian-mac-10.7.0-amd64-netinst.iso.torrent";
    let mut file = fs::File::open(filename).expect("unable to read file");
    println!("{:?}", file);
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    file.read(&mut buffer).expect("buffer overflow");
    let t = match de::from_bytes::<bencode::Torrent>(&buffer) {
        Ok(t) => t,
        Err(e) => panic!("Error: {:?}", e),
    };
    bencode::render_torrent(&t);
    println!("{:?}", t.info.pieces.len());
}
