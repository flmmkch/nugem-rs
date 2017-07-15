use std::path::PathBuf;
use std::env;

pub struct Config {
    data_paths: Vec<PathBuf>,
    window_size: (u32, u32),
    fullscreen: bool,
}

const DEFAULT_WINDOW_WIDTH : u32 = 800;
const DEFAULT_WINDOW_HEIGHT : u32 = 600;
const DEFAULT_FULLSCREEN : bool = false;

impl Config {
    pub fn new() -> Config {
        let mut data_paths = Vec::new();
        data_paths.push(PathBuf::from("./"));
        let mut window_size = (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);
        let mut fullscreen = DEFAULT_FULLSCREEN;
        // taking arguments into account
        {
            let args : Vec<_> = env::args().collect();
            let mut i : usize = 0;
            while i < args.len() {
                match args[i].as_str() {
                    "--fullscreen" => fullscreen = true,
                    "--width" => {
                        let next = &args[i+1];
                        match next.parse::<u32>() {
                            Ok(v) => {
                                window_size.0 = v;
                                i += 1;
                            }
                            _ => (),
                        }
                    },
                    "--height" => {
                        let next = &args[i+1];
                        match next.parse::<u32>() {
                            Ok(v) => {
                                window_size.1 = v;
                                i += 1;
                            }
                            _ => (),
                        }
                    }
                    "--data" => {
                        let next = &args[i+1];
                        data_paths.push(PathBuf::from(next));
                        i += 1;
                    }
                    _ => (),
                }
                i += 1;
            }
        }
        Config {
            data_paths,
            window_size,
            fullscreen,
        }
    }
    pub fn data_paths(&self) -> &[PathBuf] {
        &self.data_paths[..]
    }
    pub fn window_size(&self) -> (u32, u32) {
        self.window_size
    }
    pub fn fullscreen(&self) -> bool {
        self.fullscreen
    }
}