use anstream::println;
use anstyle::{Color, RgbColor, Style};
use clap::Parser;
use command::Args;
use ignore::{
    types::{self, TypesBuilder},
    Walk, WalkBuilder,
};
use regex::Regex;
use std::io::Result as IoResult;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Error as IoError, Read},
    path::Path,
};
mod command;
fn main() {
    let args = Args::parse();
    run(args);
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let Args {
        path,
        name,
        content,
        max_depth,
        threads,
        ignore_case,
    } = args;
    let mut types_builder = TypesBuilder::new();
    // 添加文件名的glob模式匹配
    if let Some(name) = name {
        types_builder.add("test", &name).unwrap();
        types_builder.select("all");
    };
    let types = types_builder.build().unwrap();
    //内容支持正则匹配
    let regex = Regex::new(&content).expect("build regex error");
    let walk = WalkBuilder::new(path).types(types).build();
    for results in walk {
        match results {
            Ok(entry) => {
                let metadata = entry.metadata()?;
                if metadata.is_dir() {
                    continue;
                } else if metadata.is_file() {
                    println!("{}", entry.path().display());
                    search_content(&regex, entry.path()).unwrap();
                } else {
                    continue;
                }
            }
            Err(e) => return Err(Box::new(e)),
        }
    }
    Ok(())
}

fn search_content(pattern: &Regex, path: &Path) -> IoResult<()> {
    println!("keyword is {}", pattern.as_str());
    let file = File::open(path)?;
    let mut read = BufReader::new(file);
    let mut buffer = String::new();
    let green = anstyle::Style::new().fg_color(Some(Color::Rgb(RgbColor(0, 255, 0))));
    let blue = anstyle::Style::new().fg_color(Some(Color::Rgb(RgbColor(0, 0, 255))));
    //行号
    let mut line_num = 0;
    loop {
        line_num += 1;
        match read.read_line(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                if pattern.is_match(&buffer) {
                    println!("{}{}\t{}{}", green, line_num, blue, buffer);
                }
                buffer.clear();
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
