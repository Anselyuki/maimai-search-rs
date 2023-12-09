# maimai-search-rs

使用 maimaidxprober 的 json 数据的命令行小工具

> 数据来源[舞萌 DX 查分器](https://github.com/Diving-Fish/maimaidx-prober)，感谢大佬提供的 API 接口与数据

主要的功能是查找歌曲的难度，以及查找难度的歌曲,由于本人是 Rust 初学者，故从自己的需求入手写一个小工具，仅支持命令行模式请求

## 注意事项

项目使用了 Tantivy 搜索引擎

> Tantivy是Rust实现的本地搜索库，功能对标 lucene，该库的优点在于纯 Rust 实现，性能高(lucene 的2-3倍)，资源占用低，社区活跃。

在 MacOS 与 Linux/UNIX 平台上遵守 XDG 规范，数据库与配置文件均放置于 `~/.config/maimai-search`
路径下

可以选择把本程序放置于 PATH 下，或者在使用时指定路径

## 主要功能

### 更新歌曲数据

> 只要在使用前运行一次即可，不需要每次都运行

```bash
maimai-search update
```

### B50 图片绘制

这部分复刻了 [mai-bot](https://github.com/Diving-Fish/mai-bot) 的图片绘制功能,将 Python 的`Pillow`库替换为了 Rust
的`images`库与`imageproc`库,以此实现了绘制性能的提升

![B50](docs/b50_simple.png)

> 生成这张图片的示例代码在`examples/b50.rs`中

### 歌曲查询

#### 模糊查询

可以输入歌曲部分名称查询歌曲的简单信息

> 该查询支持简繁自动转换

```bash
maimai-search 生命
```

#### ID查询

在已知某首歌ID的情况下可以使用ID精确查询。支持多个ID检索

```bash
maimai-search id 11311 11571
```

#### 详细查询

通过添加`-d`参数可以输出歌曲的铺面信息

```bash
maimai-search id 11311 -d
```

### Markdown格式输出

通过添加`md`参数可以将歌曲信息输出为 Markdown 表格

```bash
maimai-search md id 11311 -d
```

## TODO:

### 推分 list

添加推分 list 功能，可以将自己的推分列表导入到数据库中

### 计算器 Calculator

添加铺面分值计算器，可以计算歌曲的Tap、Break等各种Notes的分值