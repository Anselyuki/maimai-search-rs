use std::fs::File;
use std::io::Write;
use std::process::exit;

use log::{error, info, warn};
use schemars::schema::RootSchema;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::CONFIG_PATH;

/// 配置文件解析结果
#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    pub remote: RemoteConfig,
    pub markdown: MarkdownConfig,
}

/// 远程配置
#[derive(Serialize, Deserialize, Debug)]
pub struct RemoteConfig {
    pub json_url: String,
    pub resource_url: String,
}

/// markdown 配置
#[derive(Serialize, Deserialize, Debug)]
pub struct MarkdownConfig {
    pub picture: PictureConfig,
}

/// 远程配置
#[derive(Serialize, Deserialize, Debug)]
pub struct PictureConfig {
    pub local: bool,
    pub prefix_url: String,
}

impl Profile {
    /// 创建默认配置文件
    ///
    /// 详细的默认配置文件可以参考私有方法:`Profile::default_profile()`
    pub fn create_default() {
        let path = &CONFIG_PATH.join("config.yml");
        // 将 profile 序列化为 YAML 字符串
        let yaml = serde_yaml::to_string(&Self::default_profile()).unwrap();
        // 打开文件并写入 yaml 字符串
        let mut file = match File::create(path) {
            Ok(file) => file,
            Err(e) => {
                error!("无法创建文件: {:?}", e);
                exit(exitcode::IOERR);
            }
        };
        match file.write_all(yaml.as_bytes()) {
            Ok(_) => {
                info!("已成功创建配置文件:{}",path.display());
                open::that(path).unwrap();
            }
            Err(e) => {
                error!("无法写入文件{:?}", e);
                exit(exitcode::IOERR);
            }
        }
    }

    /// 加载配置文件,默认配置文件为`config.yml`
    ///
    /// > 如果想要创建默认配置文件,请使用`Profile::create_default()`方法
    ///
    /// - 不会抛出异常,最坏的情况下也会返回默认配置文件
    /// - 如果指定的配置文件不存在或解析失败,会产生警告信息提示配置文件配置不正确
    pub(crate) fn new() -> Profile where Profile: DeserializeOwned {
        let path = &CONFIG_PATH.join("config.yml");
        if !path.exists() { return Self::default_profile(); }

        // 通过 std::fs 读取配置文件内容,解析失败也返回默认配置文件
        let yaml_value = match std::fs::read_to_string(path) {
            Ok(file_str) => file_str,
            Err(error) => return Self::error_handler(error.to_string())
        };

        // 2.通过 serde_yaml 解析读取到的 yaml 配置转换成 json 对象
        return match serde_yaml::from_str::<RootSchema>(&yaml_value) {
            Ok(root_schema) => {
                // 通过 serde_json 把 json 对象转换指定的 model
                let data = match serde_json::to_string_pretty(&root_schema) {
                    Ok(data) => data,
                    Err(error) => return Self::error_handler(error.to_string())
                };
                match serde_json::from_str::<Profile>(&*data) {
                    Ok(profile) => profile,
                    Err(error) => Self::error_handler(error.to_string())
                }
            }
            Err(error) => Self::error_handler(error.to_string())
        };
    }

    /// 处理失败处理,返回默认配置文件
    fn error_handler(error: String) -> Profile {
        warn!("配置文件解析失败,使用默认值\n[Cause]: {}", error);
        Self::default_profile()
    }

    /// 默认配置文件的字段
    fn default_profile() -> Profile {
        Profile {
            remote: RemoteConfig {
                json_url: "https://www.diving-fish.com/api/maimaidxprober/music_data".to_string(),
                resource_url: "https://www.diving-fish.com/maibot/static.zip".to_string(),
            },
            markdown: MarkdownConfig {
                picture: PictureConfig {
                    local: false,
                    prefix_url: "https://www.diving-fish.com/covers/".to_string(),
                },
            },
        }
    }
}