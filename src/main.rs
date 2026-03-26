use anyhow::{Context, Result, anyhow};
use image::{
    ImageBuffer, Luma,
    imageops::{self},
};
use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use walkdir::WalkDir;

const PORTRAIT_SIZE: (u32, u32) = (120, 120);
const PORTRAIT_X_POS_L: u32 = 245;
const PORTRAIT_X_POS_R: u32 = 745;
const PORTRAIT_Y_POS: u32 = 580;
const NAME_TAG_Y_POS: u32 = 674;

enum PlayerSide {
    Right,
    Left,
    NotFound,
}

struct Config {
    ffmpeg: PathBuf,
    clips: PathBuf,
    frames: PathBuf,
    template: PathBuf,
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => eprintln!("Error while processing: {:?}", e),
    }
}

fn run() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    let config = parse_args(&args)?;
    let extract_frames = true;

    //let a = "-v error -select_streams v -show_entries stream=width,height -of csv=p=0:s=x";

    for (count, entry) in WalkDir::new(config.clips)
        .into_iter()
        .filter_map(|e| e.ok())
        .enumerate()
    {
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("mp4") {
            continue;
        }

        let def = count.to_string();

        let path_stem = match path.file_stem() {
            Some(p) => Path::new(p),
            None => Path::new(&def),
        };

        let in_file = match path.to_str() {
            Some(f) => Path::new(f),
            None => continue,
        };
        let out_file1 = config.frames.join(path_stem).to_string_lossy().to_string() + ".jpg";
        let out_file1 = Path::new(&out_file1);

        println!("In: {}", &in_file.display());
        if extract_frames && !out_file1.exists() {
            ffmpeg_extract_frames(&config.ffmpeg, in_file, out_file1)?;
            println!("Out: {}", &out_file1.display());
        }

        let player_side = match find_player_side(&config.template, out_file1) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error while finding player side: {:?}", e);
                continue;
            }
        };

        match player_side {
            PlayerSide::Left => {
                println!("Player 1 for {}", &in_file.display());
            }
            PlayerSide::Right => {
                println!("Player 2 for {}", &in_file.display());
            }
            PlayerSide::NotFound => {
                eprintln!("Player not found");
                continue;
            }
        };

        // TODO: determine character from player side
        // let character = find_character(player_side, out_file1);
    }
    Ok(())
}

fn parse_args(args: &[String]) -> Result<Config> {
    println!("{}", args.len());
    if args.len() < 5 {
        Err(anyhow!("Invalid number of arguments."))
    } else {
        Ok(Config {
            ffmpeg: PathBuf::from(&args[1]),
            clips: PathBuf::from(&args[2]),
            frames: PathBuf::from(&args[3]),
            template: PathBuf::from(&args[4]),
        })
    }
}

fn find_player_side(name_template: &Path, frame_path: &Path) -> Result<PlayerSide> {
    let mut side = PlayerSide::NotFound;
    let frame: ImageBuffer<Luma<f32>, Vec<f32>> =
        match image::open(frame_path).context("opening output file") {
            Ok(f) => f.to_luma32f(),
            Err(e) => return Err(e),
        };

    let image_result = find_template(name_template, &frame)?;
    let (min_x, min_y) = template_matching::find_extremes(&image_result).min_value_location;
    // println!("x, y: {}, {}", min_x, min_y);

    if min_y != NAME_TAG_Y_POS {
        return Ok(side);
    }

    let result_half_width = image_result.width / 2;
    if (0..=result_half_width).contains(&min_x) {
        side = PlayerSide::Left;
    } else if (result_half_width..=image_result.width).contains(&min_x) {
        side = PlayerSide::Right;
    }

    Ok(side)
}

#[allow(unused)]
fn find_character(side: PlayerSide, frame_path: &Path) -> Result<String> {
    let portrait_dimensions = PORTRAIT_SIZE;
    let portrait_pos = match side {
        PlayerSide::Left => (PORTRAIT_X_POS_L, PORTRAIT_Y_POS),
        PlayerSide::Right => (PORTRAIT_X_POS_R, PORTRAIT_Y_POS),
        PlayerSide::NotFound => return Err(anyhow!("Player not found")),
    };

    let frame = match image::open(frame_path).context("opening output file") {
        Ok(f) => f.to_rgb8(),
        Err(e) => return Err(e),
    };

    let _sub_frame = imageops::crop_imm(
        &frame,
        portrait_pos.0,
        portrait_pos.1,
        portrait_dimensions.0,
        portrait_dimensions.1,
    )
    .to_image();

    // use template matching
    // let _image_result = find_template(entry.path(), &sub_frame)?;
    // let (min_x, min_y) = template_matching::find_extremes(&image_result).min_value_location;
    // println!("Min x, y: {}, {}", min_x, min_y);
    Ok("Character".into())
}

fn find_template<'a>(
    template: &'a Path,
    frame: &'a ImageBuffer<Luma<f32>, Vec<f32>>,
) -> Result<template_matching::Image<'a>> {
    let template = match image::open(template).context("opening template file") {
        Ok(f) => f.to_luma32f(),
        Err(e) => return Err(e),
    };

    let result = template_matching::match_template(
        frame,
        &template,
        template_matching::MatchTemplateMethod::SumOfSquaredDifferences,
    );

    Ok(result)
}

fn ffmpeg_extract_frames(ffmpeg_path: &Path, in_file: &Path, out_file: &Path) -> Result<()> {
    let _output = Command::new(ffmpeg_path.join("ffmpeg"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // ffmpeg command to extract a frame from given mp4 video
        .args([
            "-i",
            in_file.to_string_lossy().to_string().as_ref(),
            "-vf",
            "select='eq(n\\,99)'", // frame at 100 (randomly chosen)
            "-fps_mode",
            "vfr",
            "-y",
            out_file.to_string_lossy().to_string().as_ref(),
        ])
        .output()?;
    // println!("{:?}", output);

    Ok(())
}
