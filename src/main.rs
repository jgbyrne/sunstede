mod config;

fn main() {
    match config::Config::from_file("./config.jacl") {
        Ok(conf) => {
            println!("{:#?}", conf);
        },
        Err(_) => {},
    }
}
