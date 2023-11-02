use crate::config::config::LAUNCH_PATH;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::process::exit;
use log::error;

pub struct FileUtils {}

impl FileUtils {
    /// 去除文件可能存在的拓展名
    pub fn remove_extension(filename: &str) -> String {
        let path = Path::new(filename);
        let stem = path.file_stem().unwrap();
        stem.to_string_lossy().to_string()
    }

    /// MD 文件名合法性处理
    ///
    /// - 如果输入是 md 文件，则原封不动的返回路径
    /// - 如果输入文件没有拓展名，则为其添加
    /// - 如果输入文件携带非 md 的扩展名，则报错
    pub fn add_md_extension(filename: String) -> PathBuf {
        let path = LAUNCH_PATH.join(filename);
        if let Some(ext) = path.extension() {
            if ext.eq("md") { return path.to_owned(); }
            error!("文件后缀不是\".md\",获取到\".{}\",可以选择不指定后缀名,或指定\".md\"后缀名",  ext.to_str().unwrap());
            exit(exitcode::USAGE);
        }
        let mut new_path = PathBuf::from(path);
        new_path.set_extension("md");
        return new_path;
    }

    /// 复制文件内容
    pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        let from_path = from.as_ref();
        let to_path = to.as_ref();
        // 打开源文件并创建目标文件
        let mut source_file = File::open(from_path)?;
        let mut dest_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(to_path)?;
        // 复制数据
        io::copy(&mut source_file, &mut dest_file)?;
        Ok(())
    }
}