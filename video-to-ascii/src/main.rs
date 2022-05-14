use asciify::AsciiBuilder;
use clap::Parser;
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use ffmpeg_next as ffmpeg;
use image::{DynamicImage, ImageBuffer};
use std::fs;
use std::io::Write;

const ASCII_TXT: &str = "../assets/ascii.txt";

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    width: usize,
    #[clap(short, long)]
    height: usize,
    #[clap(short, long)]
    image_location: String,
}

fn main() {
    ffmpeg::init().unwrap();
    let args = Args::parse();

    // make sure to overwrite old ascii
    if fs::remove_file(ASCII_TXT).is_err() {
        eprintln!("{} could not be deleted. Skipping...", ASCII_TXT);
    }

    // parse each frame into an ascii frame, then append it to a file
    // TODO: not efficient, due to fs calls every frame
    parse_video(args).unwrap();
}

fn append_ascii_txt(frame: String) -> std::io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(ASCII_TXT)?;

    write!(file, "{}", frame)?;
    Ok(())
}

fn convert_single_image(rgb_frame: Video, width: usize, height: usize) -> String {
    let frame_data = rgb_frame.data(0);
    let buffer = ImageBuffer::from_raw(rgb_frame.width(), rgb_frame.height(), frame_data.to_vec());
    let dynamic = DynamicImage::ImageRgb8(buffer.unwrap());
    AsciiBuilder::new_from_image(dynamic)
        .set_resize((width as u32, height as u32))
        .build()
}

// mostly taken from
// https://github.com/zmwangx/rust-ffmpeg/blob/master/examples/dump-frames.rs
fn parse_video(args: Args) -> Result<(), ffmpeg::Error> {
    if let Ok(mut ctx) = input(&args.image_location) {
        let video = ctx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;

        let context_decoder = ffmpeg::codec::context::Context::from_parameters(video.parameters())?;
        let mut decoder = context_decoder.decoder().video()?;

        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        )?;

        let mut current_index = 0;

        // lambda for easier access to variables
        let mut process_frame =
            |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
                let mut frame = Video::empty();
                while decoder.receive_frame(&mut frame).is_ok() {
                    let mut rgb_frame = Video::empty();
                    scaler.run(&frame, &mut rgb_frame)?;
                    let ascii_img = convert_single_image(rgb_frame, args.width, args.height);
                    println!("Processing frame {}", current_index);
                    append_ascii_txt(ascii_img).unwrap();
                    current_index += 1;
                }
                Ok(())
            };

        // actual iteration of frames
        let video_index = video.index();
        for (stream, packet) in ctx.packets() {
            if stream.index() == video_index {
                decoder.send_packet(&packet)?;
                process_frame(&mut decoder)?;
            }
        }
        decoder.send_eof()?;
        // last frame before eof
        process_frame(&mut decoder)?;
    }
    Ok(())
}
