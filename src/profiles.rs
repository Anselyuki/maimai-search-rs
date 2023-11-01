use std::fs::File;
use std::io::Write;

use log::{error, info};
use schemars::schema::RootSchema;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::CONFIG_PATH;

/// 配置文件解析结果
#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    pub remote: Remote,
    pub local_picture: bool,
}

/// 远程配置
#[derive(Serialize, Deserialize, Debug)]
pub struct Remote {
    pub json_url: String,
    pub resource_url: String,
}

impl Profile {
    pub fn create_default() {
        let path = &CONFIG_PATH.join("config.yml");
        let profile = Self::default_profile();

        // 将profiles序列化为YAML字符串
        let yaml = serde_yaml::to_string(&profile).unwrap();

        // 打开文件并写入yaml字符串
        let mut file = match File::create(path) {
            Ok(file) => file,
            Err(e) => panic!("Error creating file: {:?}", e),
        };
        match file.write_all(yaml.as_bytes()) {
            Ok(_) => {
                info!("已成功写入文件");
                if let Err(error) = open::that(path) {
                    error!("无法打开文件: {:?}", error);
                }
            }
            Err(e) => panic!("Error writing to file: {:?}", e),
        }
    }
    /// 加载指定配置文件
    ///
    /// 不会抛出异常,即使配置文件不存在或者解析失败
    /// 如果配置文件不存在或解析失败,会产生警告信息提示配置文件配置不正确
    pub(crate) fn new() -> Profile where Profile: DeserializeOwned {
        let path = &CONFIG_PATH.join("config.yml");
        // 配置文件不存在则返回默认配置文件
        if !path.exists() { return Self::default_profile(); }
        // 通过 std::fs 读取配置文件内容,解析失败也返回默认配置文件
        let yaml_value = match std::fs::read_to_string(path) {
            Ok(file_str) => file_str,
            Err(error) => {
                println!("配置文件解析失败,使用默认值\n[Cause]: {}", error);
                return Self::default_profile();
            }
        };

        // 2.通过serde_yaml解析读取到的yaml配置转换成json对象
        match serde_yaml::from_str::<RootSchema>(&yaml_value) {
            Ok(root_schema) => {
                // 通过serde_json把json对象转换指定的model
                let data = match serde_json::to_string_pretty(&root_schema) {
                    Ok(data) => data,
                    Err(error) => {
                        println!("解析 RootSchema 失败,使用默认值\n[Cause]: {}", error);
                        return Self::default_profile();
                    }
                };
                match serde_json::from_str::<Profile>(&*data) {
                    Ok(profile) => profile,
                    Err(error) => {
                        println!("解析 Profile 失败,使用默认值\n[Cause]: {}", error);
                        return Self::default_profile();
                    }
                }
            }
            _ => Self::default_profile()
        }
    }
    fn default_profile() -> Profile {
        Profile {
            remote: Remote {
                json_url: "https://www.diving-fish.com/api/maimaidxprober/music_data".to_string(),
                resource_url: "https://www.diving-fish.com/maibot/static.zip".to_string(),
            },
            local_picture: false,
        }
    }
}