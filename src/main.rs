use clap::Parser;
use image::{io::Reader, GrayImage, ImageBuffer, Luma};

#[derive(Parser, Debug)]
struct Args {
    /// File that is an image that should be processed
    #[arg(short, long)]
    pub file: String,

    /// White sensitivity in range from 0 (black) to 255 (white)
    #[arg(short, long, default_value_t = 20)]
    pub sensitivity: u8,

    /// Optional name for the file that is output. Requires extension.
    #[arg(long)]
    pub output_name: Option<String>,

    /// Whether to output a processed image that is high contrast
    /// It is in format of the <current_file_name>-starred.jpg
    #[arg(short, long)]
    pub output_image: bool,
}

fn main() {
    let args = Args::parse();
    let img = Reader::open(&args.file)
        .unwrap()
        .decode()
        .unwrap()
        .grayscale()
        .to_luma8();
    let (width, height) = img.dimensions();

    // Parse it to an array of bools, easier to work with
    let mut stars: Vec<Vec<bool>> = vec![vec![false; height as usize]; width as usize];
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            stars[x as usize][y as usize] = is_white(pixel, args.sensitivity);
        }
    }

    let res = count_groups(&stars);
    println!("Found {} stars", res);
    if args.output_image {
        println!("Processing into output...");
        let output = convert_to_image(&stars);
        let output_file_name = if let Some(output_name) = args.output_name {
            OutputFileName::Custom(output_name)
        } else {
            OutputFileName::FromOriginal(args.file)
        };
        output.save(create_output_path(output_file_name)).unwrap();
        println!("Done!");
    }
}

fn is_white(pixel: &Luma<u8>, sensitivity: u8) -> bool {
    pixel.0[0] > sensitivity
}

fn count_groups(stars: &Vec<Vec<bool>>) -> u64 {
    let width = stars.len();
    let height = stars[0].len();
    let mut visited = vec![vec![false; height]; width];
    let mut groups = 0;

    for y in 0..height {
        for x in 0..width {
            if stars[x][y] && !visited[x][y] {
                groups += 1;
                // println!("Group found at {} {}", x, y);
                mark_group((x, y), stars, &mut visited);
            }
        }
    }
    assert_eq!(stars, &visited, "Haven't visited all the stars!");
    groups
}

fn mark_group(start: (usize, usize), stars: &Vec<Vec<bool>>, visited: &mut Vec<Vec<bool>>) {
    let mut to_visit = vec![start];
    visited[start.0][start.1] = true;
    while let Some((x, y)) = to_visit.pop() {
        for offset_x in -1..=1 {
            let Some(new_x) = x.checked_add_signed(offset_x) else { continue };
            for offset_y in -1..=1 {
                let Some(new_y) = y.checked_add_signed(offset_y) else { continue };
                if let Some(true) = stars.get(new_x).and_then(|col| col.get(new_y)) {
                    if !visited[new_x][new_y] {
                        visited[new_x][new_y] = true;
                        to_visit.push((new_x, new_y));
                    }
                }
            }
        }
    }
}

fn convert_to_image(stars: &Vec<Vec<bool>>) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let width = stars.len();
    let height = stars[0].len();
    let mut luma = GrayImage::new(width as u32, height as u32);

    for y in 0..height {
        for x in 0..width {
            if stars[x][y] {
                let pixel = luma.get_pixel_mut(x as u32, y as u32);
                pixel.0[0] = 255;
            }
        }
    }

    luma
}

enum OutputFileName {
    FromOriginal(String),
    Custom(String),
}

fn create_output_path(output_file_name: OutputFileName) -> String {
    match output_file_name {
        OutputFileName::FromOriginal(original_file_name) => {
            original_file_name
                .split_once(".")
                .expect("File does not contain file extension")
                .0
                .to_string()
                + "-starred.jpg"
        }
        OutputFileName::Custom(custom_file_name) => custom_file_name,
    }
}
