use std::{io::Read, path::PathBuf, pin::pin, time::Duration};

use futures_core::stream::Stream;
use unitree_lidar_l1_rust::lidar::reader::{LidarReader, LidarResult};

const PATH: &str = "/dev/ttyACM0";
const BAUD_RATE: u32 = 4_000_000;

// fn main() {
//     let mut reader = LidarReader::new("/dev/ttyACM0".to_owned(), 0.0, 50.0).unwrap();
//     reader.start_lidar();
//     let mut stream = pin!(reader.into_stream());
// let t: StreamExt;
//     // pollster::block_on(async {
//     while let Some(result) = pollster::block_on(stream.) {
//         match result {
//             LidarResult::PointCloud(points) => {
//                 println!("Point cloud: {:?}", points);
//             }
//             LidarResult::ImuReading(imu) => {
//                 println!("IMU reading: {:?}", imu);
//             }
//         }
//     }
//     // });
// }

fn main() {
    // let path = PathBuf::from("/dev/ttyACM0");
    // let mut stream = OpenOptions::new()
    //     .read(true)
    //     .write(true)
    //     .open(path)
    //     .unwrap();

    let mut port = serialport::new(PATH, BAUD_RATE)
        .timeout(Duration::from_hours(1))
        // .timeout(Duration::from_millis(1000))
        .open()
        .expect("Failed to open port");

    let mut buf = vec![0; 65536];
    loop {
        let count = port.read(&mut buf).unwrap();
        if count == 0 {
            println!("LIDAR disconnected");
            break;
        }

        let buf = &buf[..count];
        println!("{}: {buf:?}", buf.len());
    }
}
