use std::fs::{create_dir_all, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{fs, io};

use log::error;
use rusttype::Font;

use crate::config::consts::{CONFIG_PATH, LAUNCH_PATH};

/// 如果路径存在则创建
pub fn create_dir(path: &PathBuf) {
    if !path.exists() {
        if let Err(error) = create_dir_all(path) {
            error!("创建文件/文件夹失败!\n[Cause]:{:?}", error)
        }
    }
}
/// 去除文件可能存在的拓展名
pub fn remove_extension(filename: String) -> String {
    let path = Path::new(&filename);
    let stem = path.file_stem().unwrap();
    stem.to_string_lossy().to_string()
}

/// 删除文件夹中的所有内容
pub fn delete_folder_contents(folder_path: &Path) -> io::Result<()> {
    // 检查指定路径是否为文件夹
    if folder_path.is_dir() {
        // 遍历目录中的所有文件和子文件夹
        for entry in fs::read_dir(folder_path)? {
            let file = entry?.path();
            if file.is_dir() {
                // 递归地删除子文件夹中的所有文件
                delete_folder_contents(&file)?;
                // 删除空的子文件夹
                fs::remove_dir(file)?;
            } else {
                // 删除文件
                fs::remove_file(file)?;
            }
        }
    }
    Ok(())
}

/// MD 文件名合法性处理
///
/// - 如果输入是 md 文件，则原封不动的返回路径
/// - 如果输入文件没有拓展名，则为其添加
/// - 如果输入文件携带非 md 的扩展名，则报错
pub fn add_md_extension(filename: String) -> PathBuf {
    let path = LAUNCH_PATH.join(filename);
    if let Some(ext) = path.extension() {
        if ext.eq("md") {
            return path.to_owned();
        }
        error!(
            "文件后缀不是\".md\",获取到\".{}\",可以选择不指定后缀名,或指定\".md\"后缀名",
            ext.to_str().unwrap()
        );
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

/// 获取微软雅黑字体
pub fn get_msyh_font() -> Font<'static> {
    let path = CONFIG_PATH.join("resource").join("msyh.ttc");
    let font_data = fs::read(path).unwrap();
    let font = Font::try_from_bytes(font_data.leak()).unwrap();
    return font;
}

/// 获取 Adobe 黑体字体
pub fn get_adobe_simhei_font() -> Font<'static> {
    let path = CONFIG_PATH.join("resource").join("adobe_simhei.otf");
    let font_data = fs::read(path).unwrap();
    Font::try_from_bytes(font_data.leak()).unwrap()
}
