mod client;
use client::LeechClient;

use client::tracker::TrackerRequest;

// use std::fs;
// use std::io::Read;

use reqwest;

extern crate serde_derive;

#[tokio::main]
async fn main() {
    let filename = "debian-10.8.0-amd64-netinst.iso.torrent";
    let client = LeechClient::new(filename);
    println!("{:?}", client.torrent_file.info.info_hash);
    // let info_hash = client
    //     .torrent_file
    //     .info
    //     .info_hash
    //     .iter()
    //     .map(|v| format!("{:X?}", v))
    //     .collect::<Vec<String>>();
    // println!("{:?}", info_hash);
    // let params = [
    //     ("info_hash".to_string(), info_hash),
    //     (
    //         "peer_id".to_string(),
    //         String::from_utf8_lossy(&[
    //             1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    //         ])
    //         .to_string(),
    //     ),
    //     ("port".to_string(), "6881".to_string()),
    //     ("uploaded".to_string(), "0".to_string()),
    //     ("downloaded".to_string(), "0".to_string()),
    //     ("compact".to_string(), "1".to_string()),
    //     (
    //         "left".to_string(),
    //         format!("{}", client.torrent_file.info.length.unwrap()),
    //     ),
    // ];
    let track_req = TrackerRequest::from(&client.torrent_file);
    println!("qp: {}", track_req);
    // let http = reqwest::Client::new();
    let res = reqwest::get(&track_req.to_string()).await.expect("cool");
    println!("status: {}", res.status());
    let body = res.text().await.expect("text");
    println!("body: {}", body);

    // println!("response: {:?}", res);
}
