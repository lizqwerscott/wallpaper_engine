use std::env;
use std::path::Path;

use wallpaper_engine::{generate_wallpapers, load_wallpaper, play_wallpapers};

fn main() {
    let args: Vec<String> = env::args().collect();

    let mutep = &args[1] == "mute";

    let dir_path = Path::new(&args[2]);

    // 加载壁纸
    let res = load_wallpaper(dir_path).unwrap();

    for (index, wallpaper) in res.iter().enumerate() {
        println!(
            "{}: title: {}, file: {}",
            index,
            wallpaper.title,
            wallpaper.file.display()
        );
    }

    if args.len() == 4 {
        let output_path = Path::new(&args[3]);
        generate_wallpapers(&res, output_path).unwrap();
    } else {
        // 按照顺序播放壁纸
        play_wallpapers(dir_path, &res, mutep).unwrap();
    }
}
