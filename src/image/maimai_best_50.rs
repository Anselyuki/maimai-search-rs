use std::collections::HashMap;
use std::path::PathBuf;

use image::{DynamicImage, GenericImageView, ImageError, ImageFormat, RgbaImage};
use image::imageops::{FilterType, overlay};
use imageproc::drawing::{Canvas, draw_text_mut};
use rusttype::Scale;

use crate::clients::user_data::entity::ChartInfoResponse;
use crate::config::consts::{CONFIG_PATH, LAUNCH_PATH};
use crate::image::utils::{compute_ra, get_ra_pic, string_to_half_width};
use crate::utils::file::FileUtils;

static WIDTHS: [(u32, i32); 38] = [
    (126, 1),
    (159, 0),
    (687, 1),
    (710, 0),
    (711, 1),
    (727, 0),
    (733, 1),
    (879, 0),
    (1154, 1),
    (1161, 0),
    (4347, 1),
    (4447, 2),
    (7467, 1),
    (7521, 0),
    (8369, 1),
    (8426, 0),
    (9000, 1),
    (9002, 2),
    (11021, 1),
    (12350, 2),
    (12351, 1),
    (12438, 2),
    (12442, 0),
    (19893, 2),
    (19967, 1),
    (55203, 2),
    (63743, 1),
    (64106, 2),
    (65039, 1),
    (65059, 0),
    (65131, 2),
    (65279, 1),
    (65376, 2),
    (65500, 1),
    (65510, 2),
    (120831, 1),
    (262141, 2),
    (1114109, 1),
];

static COLOR: [(i32, i32, i32); 5] = [
    (69, 193, 36),
    (255, 186, 1),
    (255, 90, 102),
    (134, 49, 200),
    (217, 197, 233),
];

static COLUMNS_IMG: [i32; 12] = [2, 140, 278, 416, 554, 692, 830, 968, 988, 1126, 1264, 1402];
static ROWS_IMG: [i32; 6] = [116, 212, 308, 404, 500, 596];

static COLUMNS_RATING: [i64; 5] = [86, 100, 115, 130, 145];

/// # 自排序 Best 列表
///
/// `ChartInfo` 实现了基于歌曲 RATING 的排序比较规则,根据这个规则实现了一个插入时排序的列表用于安放
///
/// - `BestList::new(15)` 可以创建一个大小为 15 的列表,用来装载 B15
/// - `BestList::new(35)` 可以创建一个大小为 35 的列表,用来装载 B35
///
/// > 这个结构体不知道要不要留,先按照 [mai-bot](https://github.com/Diving-Fish/mai-bot) 的规则来
///
/// 毕竟*最大头的还是PIL库的调用*
#[derive(Debug)]
pub struct BestList {
    data: Vec<ChartInfoResponse>,
    size: usize,
}

impl BestList {
    pub fn new(size: usize) -> Self {
        Self { data: vec![], size }
    }

    pub fn push(&mut self, elem: ChartInfoResponse) {
        if self.data.len() >= self.size && elem < *self.data.last().unwrap() {
            return;
        }
        self.data.push(elem);
        self.data.sort();
        self.data.reverse();
        while self.data.len() > self.size {
            self.data.pop();
        }
    }

    pub fn pop(&mut self) -> Option<ChartInfoResponse> {
        self.data.pop()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl std::fmt::Display for BestList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let data_str = self
            .data
            .iter()
            .map(|ci| format!("\t{:?}\n", ci))
            .collect::<String>();
        write!(f, "[\n{}\n]", data_str)
    }
}

impl std::ops::Index<usize> for BestList {
    type Output = ChartInfoResponse;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

/// # 绘图库实现类
///
/// 这里面有一个或者多个函数要用 pyo3 进行调用
#[derive(Debug)]
pub struct DrawBest {
    /// B35 列表
    sd_best: BestList,
    /// B15 列表
    dx_best: BestList,
    /// 用户名(maimai DX)
    username: String,
    /// 标准谱面 Rating
    sd_rating: i32,
    /// DX 谱面 Rating
    dx_rating: i32,
    /// 用户 Rating(SD + DX)
    player_rating: i32,
    /// 图片目录
    pic_dir: PathBuf,
    /// 封面目录
    cover_dir: PathBuf,
    /// 基底图片,可以理解为画布
    img: RgbaImage,
}

impl DrawBest {
    /// 初始化绘图类
    ///
    /// 对应 Python 脚本里的 `__init__` 函数
    pub fn new(sd_best: BestList, dx_best: BestList, username: &str) -> Self {
        // 计算标准谱面的 Rating
        let sd_rating: i32 = sd_best
            .data
            .iter()
            .map(|sd| compute_ra(sd.ds, sd.achievements))
            .sum();
        // 计算 DX 谱面的 Rating
        let dx_rating: i32 = dx_best
            .data
            .iter()
            .map(|sd| compute_ra(sd.ds, sd.achievements))
            .sum();

        let img_path = CONFIG_PATH
            .join("resource")
            .join("mai")
            .join("pic")
            .join("UI_TTR_BG_Base_Plus.png");
        DrawBest {
            sd_best,
            dx_best,
            username: string_to_half_width(username),
            sd_rating,
            dx_rating,
            player_rating: sd_rating + dx_rating,
            pic_dir: CONFIG_PATH.join("resource").join("mai").join("pic"),
            cover_dir: CONFIG_PATH.join("resource").join("mai").join("cover"),
            img: image::open(img_path).unwrap().to_rgba8(),
        }
    }

    /// 对应 Python 脚本里的 `_getCharWidth` 函数
    fn get_char_width(&self, o: u32) -> i32 {
        match o {
            _ => {}
        }
        if o == 0xe || o == 0xf {
            return 0;
        }
        for &(num, wid) in WIDTHS.iter() {
            if o <= num {
                return wid;
            }
        }
        return 1;
    }

    /// 对应 Python 脚本里的 `_columnWidth` 函数(这函数是不是打错字了)
    fn column_width(&self, s: &str) -> i32 {
        s.chars().map(|ch| self.get_char_width(ch as u32)).sum()
    }

    /// 对应 Python 脚本里的 `_changeColumnWidth` 函数,并进行了优化
    ///
    /// > 优化后的代码使用了一个哈希表来缓存 `get_char_width()` 方法的结果,这样可以避免在每个字符上重复调用该方法
    fn change_column_width(&self, s: &str, len: i32) -> String {
        let mut res = 0;
        let mut char_list = Vec::new();
        let mut char_width_cache = HashMap::new();

        for ch in s.chars() {
            res += *char_width_cache
                .entry(ch)
                .or_insert_with(|| self.get_char_width(ch as u32));
            if res <= len {
                char_list.push(ch);
            } else {
                break;
            }
        }

        let mut output_str = String::with_capacity(char_list.len());
        output_str.extend(char_list);
        output_str
    }

    /// # 缩放图片
    ///
    /// 将大小不等的图片缩放指定的比例
    fn resize_pic(mut image: &DynamicImage, time: f32) -> DynamicImage {
        let width = f32::floor(image.width() as f32 * time) as u32;
        let height = f32::floor(image.height() as f32 * time) as u32;
        image.resize(width, height, FilterType::Lanczos3)
    }

    /// # 绘制 Rating 数字
    ///
    /// 在图片上绘制 Rating 数字
    fn draw_rating(&self, mut rating_base_img: DynamicImage) -> DynamicImage {
        let num_str = self.player_rating.to_string();
        let mut digits: Vec<char> = num_str.chars().collect();
        for (digit, index) in digits.iter().rev().zip(COLUMNS_RATING.iter().rev()) {
            let mut digit_img =
                image::open(self.pic_dir.join(format!("UI_NUM_Drating_{}.png", digit))).unwrap();
            digit_img = Self::resize_pic(&digit_img, 0.6);
            overlay(&mut rating_base_img, &digit_img, index - 2, 9);
        }
        return rating_base_img;
    }

    ///TODO: 代码最多的一集.jpg
    fn draw_best_list(&mut self) {
        let item_w = 131;
        let item_h = 88;
        let level_triangle = [(item_w, 0), (item_w - 27, 0), (item_w, 27)];
    }

    pub fn draw(&mut self) -> Result<(), ImageError> {
        // Splash LOGO
        let mut splash_logo =
            image::open(self.pic_dir.join("UI_CMN_TabTitle_MaimaiTitle_Ver214.png"))?;
        splash_logo = Self::resize_pic(&splash_logo, 0.65);
        overlay(&mut self.img, &splash_logo, 10, 10);

        // 绘制 Rating 数字
        let mut rating_base_img =
            image::open(self.pic_dir.join(get_ra_pic(self.player_rating as u32)))?;
        rating_base_img = self.draw_rating(rating_base_img);
        rating_base_img = Self::resize_pic(&rating_base_img, 0.85);
        overlay(&mut self.img, &rating_base_img, 240, 8);

        // 绘制姓名列
        let mut name_plate_img = image::open(self.pic_dir.join("UI_TST_PlateMask.png"))?;
        name_plate_img = name_plate_img.resize_exact(272, 40, FilterType::Lanczos3);
        draw_text_mut(
            &mut name_plate_img,
            image::Rgba([0, 0, 0, 255]),
            10,
            5,
            Scale::uniform(28.0),
            &FileUtils::get_msyh_font(),
            &self.username,
        );

        let mut name_dx_img = image::open(self.pic_dir.join("UI_CMN_Name_DX.png"))?;
        name_dx_img = Self::resize_pic(&name_dx_img, 0.9);

        overlay(&mut name_plate_img, &name_dx_img, 220, 4);
        overlay(&mut self.img, &name_plate_img, 240, 40);

        // 姓名列下面的 DX 分数计算列
        let mut shougou_img = image::open(self.pic_dir.join("UI_CMN_Shougou_Rainbow.png"))?;
        let play_count_info = format!(
            "SD: {} + DX: {} = {}",
            self.sd_rating, self.dx_rating, self.player_rating
        );
        let shougou_img_w = shougou_img.width();
        let shougou_img_h = shougou_img.height();

        draw_text_mut(
            &mut shougou_img,
            image::Rgba([0, 0, 0, 255]),
            12,
            4,
            Scale::uniform(14.0),
            &FileUtils::get_adobe_simhei_font(),
            &play_count_info,
        );

        overlay(&mut self.img, &shougou_img, 240, 83);

        //playCountInfoW, playCountInfoH = shougouDraw.textsize(playCountInfo, font2)
        //textPos = ((shougouImgW - playCountInfoW - font2.getoffset(playCountInfo)[0]) / 2, 5)
        //shougouDraw.text((textPos[0] - 1, textPos[1]), playCountInfo, 'black', font2)
        //shougouDraw.text((textPos[0] + 1, textPos[1]), playCountInfo, 'black', font2)
        //shougouDraw.text((textPos[0], textPos[1] - 1), playCountInfo, 'black', font2)
        //shougouDraw.text((textPos[0], textPos[1] + 1), playCountInfo, 'black', font2)
        //shougouDraw.text((textPos[0] - 1, textPos[1] - 1), playCountInfo, 'black', font2)
        //shougouDraw.text((textPos[0] + 1, textPos[1] - 1), playCountInfo, 'black', font2)
        //shougouDraw.text((textPos[0] - 1, textPos[1] + 1), playCountInfo, 'black', font2)
        //shougouDraw.text((textPos[0] + 1, textPos[1] + 1), playCountInfo, 'black', font2)
        //shougouDraw.text(textPos, playCountInfo, 'white', font2)
        //shougouImg = self._resizePic(shougouImg, 1.05)
        //self.img.paste(shougouImg, (240, 83), mask=shougouImg.split()[3])

        self.draw_best_list();

        let author_board_img_path = self.pic_dir.join("UI_CMN_MiniDialog_01.png");
        dbg!(&author_board_img_path);
        assert!(author_board_img_path.exists());
        //authorBoardImg = Image.open(self.pic_dir + 'UI_CMN_MiniDialog_01.png').convert('RGBA')
        //authorBoardImg = self._resizePic(authorBoardImg, 0.35)
        //authorBoardDraw = ImageDraw.Draw(authorBoardImg)
        //authorBoardDraw.text((31, 28), '   Generated By\nXybBot & AnselYuki', 'black', font2)
        //self.img.paste(authorBoardImg, (1224, 19), mask=authorBoardImg.split()[3])

        let dx_img_path = self.pic_dir.join("UI_RSL_MBase_Parts_01.png");
        dbg!(&dx_img_path);
        assert!(dx_img_path.exists());
        //dxImg = Image.open(self.pic_dir + 'UI_RSL_MBase_Parts_01.png').convert('RGBA')
        //self.img.paste(dxImg, (988, 65), mask=dxImg.split()[3])

        let sd_img_path = self.pic_dir.join("UI_RSL_MBase_Parts_02.png");
        dbg!(&sd_img_path);
        assert!(sd_img_path.exists());
        //sdImg = Image.open(self.pic_dir + 'UI_RSL_MBase_Parts_02.png').convert('RGBA')
        //self.img.paste(sdImg, (865, 65), mask=sdImg.split()[3])

        let path = LAUNCH_PATH.join("b50.png");
        self.img
            .save_with_format(&path, ImageFormat::Png)
            .expect("TODO: panic message");
        open::that(&path).unwrap();
        Ok(())
    }
}
