//! Drop: детерминированное освобождение ресурсов + std::mem::drop

use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

struct TempFile {
    path: PathBuf,
    file: File,
}

impl TempFile {
    fn new(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let file = File::create(&path)?;
        Ok(Self { path, file })
    }

    fn write_line(&mut self, line: &str) -> io::Result<()> {
        writeln!(self.file, "{line}")
    }
}

// Вызывается автоматически при выходе из области видимости
impl Drop for TempFile {
    fn drop(&mut self) {
        // Файл закрывается автоматически при Drop у File,
        // но мы покажем момент освобождения:
        eprintln!("Drop: закрываем и удаляем временный файл {:?}", self.path);
        // Попробуем удалить файл (игнорируем ошибку в демонстрационных целях)
        let _ = std::fs::remove_file(&self.path);
    }
}

fn main() -> io::Result<()> {
    {
        let mut tmp = TempFile::new("temp_demo.txt")?;
        tmp.write_line("Первая строка")?;
        tmp.write_line("Вторая строка")?;

        // Можно вручную вызвать std::mem::drop, чтобы освободить ресурс раньше:
        // std::mem::drop(tmp);
        // println!("tmp уже удалён");

        println!("Временный файл создан и записан. Скоро выйдем из области...");
    } // <- Здесь автоматически вызовется Drop у TempFile

    println!("Готово: ресурс освобождён детерминированно.");
    Ok(())
}