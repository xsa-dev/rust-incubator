use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

trait Logger {
    fn log(&self, message: &str);
}

struct ConsoleLogger;

struct FileLogger<'a> {
    path: &'a Path, // <- ссылка на Path (DST), это допустимо
}

impl Logger for ConsoleLogger {
    fn log(&self, message: &str) {
        println!("[Console] {message}");
    }
}

impl<'a> Logger for FileLogger<'a> {
    fn log(&self, message: &str) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.path)
            .expect("failed to open log file");
        writeln!(file, "[File] {message}").expect("failed to write log");
    }
}

fn run(logger: &dyn Logger) {
    logger.log("Система запущена");
    logger.log("Всё работает отлично!");
}

fn main() {
    let console = ConsoleLogger;

    let pb: PathBuf = "log.txt".into();
    let file = FileLogger { path: pb.as_path() };

    run(&console);
    run(&file);
}