# maimai-search-rs

使用 maimaidxprober 的 json 数据的命令行小工具

> 数据来源[舞萌 DX 查分器](https://github.com/Diving-Fish/maimaidx-prober)，感谢大佬提供的 API 接口与数据

## 核心功能

对于接收的部分歌曲信息进行检索,由于本人是 Rust 初学者，故从自己的需求入手写一个小工具，仅支持命令行模式请求

## 注意事项

项目使用了 SQLite 数据库，在 MacOS 与 Linux/UNIX 平台上遵守 XDG 规范，数据库与配置文件均放置于 `~/.config/maimai-search`
路径下

可以选择把本程序放置于 PATH 下，或者在使用时指定路径

### 安装

> 更新在线数据（只需更新一次即可）

#### Linux / macOS
```bash
maimai-search update
```

#### Windows
```bash
maimai-search.exe update
```
## 主要功能

### 模糊搜索
    输入希望查询的歌名的部分信息（比如 `初音` ）

```bash
maimai-search 初音
```

    显示模糊搜索可匹配到的n个结果（n为参数）

```bash
maimai-search S -c 200
```
### ID搜索/精确搜索

```bash
maimai-search id 11311
```

### 输出查询对象的铺面信息

```bash
maimai-search id 11311 -d
```

### ~~Markdown格式输出查询结果~~（重构中）

```bash
maimai-search id 11311 -m
```
## TODO:

- 推分 list
  - 添加推分 list 功能，可以将自己的推分列表导入到数据库中
- 重构Markdown功能
- 重构数据库类型
- 加入b50生成功能（利用pyo3 module）