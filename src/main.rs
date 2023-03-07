use clap::Parser;
use serde::Deserialize;
use std::borrow::Cow;
use std::{
    env,
    fs::{create_dir_all, File},
    io::{copy, Cursor},
    path::PathBuf,
    process::Command,
};
use url::Url;
#[macro_use]
extern crate log;

const BASE_URL: &str = "https://www.bing.com";
const DEFAULT_FILENAME: &str = "bwp.jpg";
const BWP_DIRNAME: &str = ".bwp";

#[derive(Parser)]
struct Cli {
    #[arg(default_value_t = String::from("ls"))]
    cmd: String,
    #[arg(short, long, default_value_t = 0)]
    num: usize,
    #[arg(short, long, default_value_t = String::from(""))]
    region: String,
    #[arg(short, long, default_value_t = false)]
    set: bool,
}

#[derive(Deserialize)]
struct BingWallpapers {
    images: Vec<BingWallpaper>,
}

#[derive(Deserialize)]
struct BingWallpaper {
    title: String,
    url: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let bwp_dir = setup_bwp_dir();
    let args = Cli::parse();
    match args.cmd.as_str() {
        "ls" => {
            println!("cmd is {}", args.cmd);
        }
        _ => {
            panic!("unknown command {}", args.cmd)
        }
    }

    debug!("getting wallpaper number: {}", args.num);
    let download_url = get_wallpaper_url(args.num, args.region)?;
    let response = reqwest::blocking::get(&download_url)?;
    let url_parsed = Url::parse(&download_url)?;
    let mut filename = String::from(DEFAULT_FILENAME);
    for (k, v) in url_parsed.query_pairs() {
        if let Cow::Borrowed("id") = k {
            filename = v.into();
        }
    }
    let mut img_bytes = Cursor::new(response.bytes()?);

    let path = bwp_dir?.join(filename);
    debug!("will be saved to: {:?}", path);
    let mut dest = File::create(&path)?;
    copy(&mut img_bytes, &mut dest)?;
    println!("saved to: {:?}", path);
    if args.set {
        set_bg(&path)?;
    }
    Ok(())
}

fn setup_bwp_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let user_dir = dirs::home_dir().unwrap();
    let bwp_dir = user_dir.join(BWP_DIRNAME);
    debug!("creating bwp directory: {:?}", bwp_dir);
    create_dir_all(&bwp_dir)?;
    Ok(bwp_dir)
}

fn get_wallpaper_url(num: usize, region: String) -> Result<String, Box<dyn std::error::Error>> {
    // TODO: extract getting wallpaper list
    let url = format!("{BASE_URL}/HPImageArchive.aspx?format=js&idx=0&n=10&mkt={region}");
    let resp = reqwest::blocking::get(url)?.text()?;
    debug!("{resp}");
    let parsed: BingWallpapers = serde_json::from_str(&resp)?;
    for (idx, image) in parsed.images.iter().enumerate() {
        println!("{}: {}", idx, image.title);
    }
    let download_url = format!("{}{}", BASE_URL, parsed.images.get(num).unwrap().url);
    info!(
        "{}: {}",
        parsed.images.get(num).unwrap().title,
        download_url
    );
    Ok(download_url)
}

fn set_bg(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    match env::consts::OS {
        "linux" => {
            Command::new("feh")
                .arg("--bg-scale")
                .arg(path)
                .output()?;
            println!("set wallpaper with feh");
        }
        _ => {
            println!("wallpaper setter not implemented for {}!", env::consts::OS);
        }
    }
    Ok(())
}
