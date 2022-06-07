//! Exif is a binary that take a JPEG or TIFF file as argument and extract it's metadata as json

extern crate tap_plugin_exif;

use std::env;
use std::fs::File;

use tap_plugin_exif::Exif;

fn main() {
  if env::args().len() != 2 
  {
    println!("exif input_path");
    return;
  }

  let args: Vec<String> = env::args().collect();
  let file_path = &args[1];

  match File::open(file_path) 
  {
    Err(_) => println!("Can't open file {}", file_path),
    Ok(file) => 
    {
      let exif = Exif {};
      if let Some(attributes) = exif.to_attributes(Box::new(file)) 
      {
        println!("{}", serde_json::to_string(&attributes).unwrap());
      }
    }
  }
}
