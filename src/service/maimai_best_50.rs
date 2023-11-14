use std::ops::Index;
use std::path::PathBuf;

use image::imageops::{overlay, FilterType};
use image::{DynamicImage, ImageError, ImageFormat, Pixel, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_polygon_mut, draw_text_mut};
use imageproc::map::map_colors_mut;
use imageproc::point::Point;
use imageproc::rect::Rect;
use rusttype::Scale;

use crate::clients::user_data::entity::ChartInfoResponse;
use crate::config::consts::{CONFIG_PATH, LAUNCH_PATH};
use crate::utils::file::FileUtils;
use crate::utils::image::{change_column_width, compute_ra, get_ra_pic, string_to_half_width};

static OFFSET: [(i32, i32); 8] = [
    (-1, -1),
    (1, -1),
    (0, -1),
    (-1, 1),
    (1, 1),
    (0, 1),
    (-1, 0),
    (1, 0),
];
static COLUMNS_RATING: [i64; 5] = [84, 98, 113, 128, 143];
static ITEM_WIDTH: i32 = 131;
static ITEM_HEIGHT: i32 = 88;
static VERTICAL_SPACING: i32 = 8;
static HORIZONTAL_SPACING: i32 = 7;

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
            .map(|ci| format!("\t{}\n", ci))
            .collect::<String>();
        write!(f, "[\n{}\n]", data_str)
    }
}

impl Index<usize> for BestList {
    type Output = ChartInfoResponse;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

/// # 绘图库实现类
///
/// 这里面有一个或者多个函数要用 pyo3 进行调用
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
    img: DynamicImage,
}

impl DrawBest {
    /// 初始化绘图
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
        DrawBest {
            sd_best,
            dx_best,
            username: string_to_half_width(username),
            sd_rating,
            dx_rating,
            player_rating: sd_rating + dx_rating,
            pic_dir: CONFIG_PATH.join("resource/mai/pic"),
            cover_dir: CONFIG_PATH.join("resource/mai/cover"),
            img: image::open(CONFIG_PATH.join("resource/mai/pic/UI_TTR_BG_Base_Plus.png")).unwrap(),
        }
    }

    /// # 缩放图片
    ///
    /// 将大小不等的图片缩放指定的比例
    fn resize_pic(image: &DynamicImage, time: f32) -> DynamicImage {
        let width = f32::floor(image.width() as f32 * time) as u32;
        let height = f32::floor(image.height() as f32 * time) as u32;
        image.resize(width, height, FilterType::Lanczos3)
    }

    /// # 绘制 Rating 数字
    ///
    /// 在图片上绘制 Rating 数字
    fn draw_rating(&self, mut rating_base_img: DynamicImage) -> DynamicImage {
        let num_str = self.player_rating.to_string();
        let digits: Vec<char> = num_str.chars().collect();
        for (digit, index) in digits.iter().rev().zip(COLUMNS_RATING.iter().rev()) {
            let mut digit_img =
                image::open(self.pic_dir.join(format!("UI_NUM_Drating_{}.png", digit))).unwrap();
            digit_img = Self::resize_pic(&digit_img, 0.6);
            overlay(&mut rating_base_img, &digit_img, *index, 9);
        }
        return rating_base_img;
    }

    /// 绘制歌曲列表
    fn draw_best_list(&mut self) -> Result<DynamicImage, ImageError> {
        // 绘制 b15 存在的图片列
        for num in 0..self.dx_best.len() {
            let column = 75i64
                + (ITEM_WIDTH * (num % 3 + 7) as i32 + HORIZONTAL_SPACING * (num % 3) as i32)
                    as i64;
            let row = 120i64
                + (ITEM_HEIGHT * (num / 3) as i32 + VERTICAL_SPACING * (num / 3) as i32) as i64;
            // 3 列一行排列的 B15
            let cover = self.draw_best_item(num, true)?;
            // 绘制 item 的阴影,并把绘制完的 item 覆盖到最终输出里
            self.img = self.draw_item_shadow(column, row);
            overlay(&mut self.img, &cover, column, row);
        }

        // 绘制 b35 存在的图片列
        for num in 0..self.sd_best.len() {
            let column = 6i64
                + (ITEM_WIDTH * (num % 7) as i32 + HORIZONTAL_SPACING * (num % 7) as i32) as i64;
            let row = 120i64
                + (ITEM_HEIGHT * (num / 7) as i32 + VERTICAL_SPACING * (num / 7) as i32) as i64;
            // 7 列一行排列的 B35
            let cover = self.draw_best_item(num, false)?;
            // 绘制 item 的阴影,并把绘制完的 item 覆盖到最终输出里
            self.img = self.draw_item_shadow(column, row);
            overlay(&mut self.img, &cover, column, row);
        }

        let mut blank_cover = image::open(self.cover_dir.join("01000.png"))?;
        blank_cover =
            Self::resize_pic(&blank_cover, ITEM_WIDTH as f32 / blank_cover.width() as f32);
        blank_cover = blank_cover.crop_imm(
            0,
            (blank_cover.height() - ITEM_HEIGHT as u32) / 2,
            ITEM_WIDTH as u32,
            ITEM_HEIGHT as u32,
        );
        blank_cover = blank_cover.blur(3.0);
        // 这里处理不完整的 b15 列表占位图
        for num in self.dx_best.len()..self.dx_best.size {
            let column = 75i64
                + (ITEM_WIDTH * (num % 3 + 7) as i32 + HORIZONTAL_SPACING * (num % 3) as i32)
                    as i64;
            let row = 120i64
                + (ITEM_HEIGHT * (num / 3) as i32 + VERTICAL_SPACING * (num / 3) as i32) as i64;
            self.img = self.draw_item_shadow(column, row);
            overlay(&mut self.img, &blank_cover, column, row);
        }
        // 这里处理不完整的 b35 列表占位图
        for num in self.sd_best.len()..self.sd_best.size {
            let column: i64 = 6i64
                + (ITEM_WIDTH * (num % 7) as i32 + HORIZONTAL_SPACING * (num % 7) as i32) as i64;
            let row: i64 = 120i64
                + (ITEM_HEIGHT * (num / 7) as i32 + VERTICAL_SPACING * (num / 7) as i32) as i64;
            self.img = self.draw_item_shadow(column, row);
            overlay(&mut self.img, &blank_cover, column, row);
        }
        Ok(self.img.clone())
    }

    /// # 绘制单个谱面元素
    ///
    /// - `new` 用于控制是绘制 B15 还是 B35 列表
    fn draw_best_item(&mut self, num: usize, new: bool) -> Result<DynamicImage, ImageError> {
        let chart = match new {
            true => self.dx_best.index(num),
            false => self.sd_best.index(num),
        };
        let level_triangle = [
            Point::new(ITEM_WIDTH, 0),
            Point::new(ITEM_WIDTH - 27, 0),
            Point::new(ITEM_WIDTH, 27),
        ];
        let font = FileUtils::get_adobe_simhei_font();

        // 获取歌曲封面
        let mut cover = match image::open(self.cover_dir.join(format!("{:0>5}.png", chart.song_id)))
        {
            Ok(image) => image,
            Err(_) => image::open(self.cover_dir.join("01000.png"))?,
        };
        cover = Self::resize_pic(&cover, ITEM_WIDTH as f32 / cover.width() as f32);
        // 裁剪谱面图片,加上高斯模糊
        cover = cover
            .crop_imm(
                0,
                (cover.height() - ITEM_HEIGHT as u32) / 2,
                ITEM_WIDTH as u32,
                ITEM_HEIGHT as u32,
            )
            .blur(3.0);
        // 谱面图片压暗
        map_colors_mut(&mut cover, |pixel| {
            let rgba = pixel.channels();
            Rgba([
                (rgba[0] as f32 * 0.72).floor() as u8,
                (rgba[1] as f32 * 0.72).floor() as u8,
                (rgba[2] as f32 * 0.72).floor() as u8,
                rgba[3],
            ])
        });
        // 在谱面右上角绘制等级定数小三角
        draw_polygon_mut(&mut cover, &level_triangle, chart.level_label.label_color());

        // 绘制谱面标题
        draw_text_mut(
            &mut cover,
            Rgba([255, 255, 255, 255]),
            8,
            8,
            Scale::uniform(16.0),
            &font,
            change_column_width(&*chart.title, ITEM_WIDTH).as_str(),
        );

        // 绘制达成率
        draw_text_mut(
            &mut cover,
            Rgba([255, 255, 255, 255]),
            7,
            28,
            Scale::uniform(12.0),
            &font,
            format!("{:.4}%", chart.achievements).as_str(),
        );

        // Rank 图片
        let mut rank_img = image::open(self.pic_dir.join(chart.rate.get_file_name()))?;
        rank_img = Self::resize_pic(&rank_img, 0.3);
        overlay(&mut cover, &rank_img, 72, 28);

        let mut blank_img = image::open(self.pic_dir.join(format!("UI_MSS_MBase_Icon_Blank.png")))?;
        blank_img = Self::resize_pic(&blank_img, 0.48);
        if !chart.fc.is_empty() {
            let mut fc_img = image::open(
                self.pic_dir
                    .join(format!("UI_MSS_MBase_Icon_{}_S.png", chart.fc)),
            )?;
            fc_img = Self::resize_pic(&fc_img, 0.48);
            overlay(&mut cover, &fc_img, 105, 60);
        } else {
            overlay(&mut cover, &blank_img, 105, 60);
        }

        if !chart.fs.is_empty() {
            let mut fs_img = image::open(
                self.pic_dir
                    .join(format!("UI_MSS_MBase_Icon_{}_S.png", chart.fs)),
            )?;
            fs_img = Self::resize_pic(&fs_img, 0.48);
            overlay(&mut cover, &fs_img, 80, 60);
        } else {
            overlay(&mut cover, &blank_img, 80, 60);
        }

        draw_text_mut(
            &mut cover,
            Rgba([255, 255, 255, 255]),
            8,
            44,
            Scale::uniform(12.0),
            &font,
            format!(
                "Base: {} -> {}",
                chart.ds,
                compute_ra(chart.ds, chart.achievements)
            )
            .as_str(),
        );
        draw_text_mut(
            &mut cover,
            Rgba([255, 255, 255, 255]),
            8,
            60,
            Scale::uniform(18.0),
            &font,
            format!("#{}", num + 1).as_str(),
        );
        Ok(cover)
    }

    /// 绘制谱面元素下面的阴影
    fn draw_item_shadow(&mut self, x: i64, y: i64) -> DynamicImage {
        let mut mask = RgbaImage::new(ITEM_WIDTH as u32, ITEM_HEIGHT as u32);
        draw_filled_rect_mut(
            &mut mask,
            Rect::at(0, 0).of_size(ITEM_WIDTH as u32, ITEM_HEIGHT as u32),
            Rgba([0, 0, 0, 150]),
        );
        overlay(&mut self.img, &mask, x + 2, y + 2);
        self.img.clone()
    }

    pub fn draw(&mut self) -> Result<(), ImageError> {
        let font = FileUtils::get_adobe_simhei_font();
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
        name_plate_img = name_plate_img.resize_exact(280, 40, FilterType::Lanczos3);

        draw_text_mut(
            &mut name_plate_img,
            Rgba([0, 0, 0, 255]),
            10,
            4,
            Scale::uniform(32.0),
            &FileUtils::get_msyh_font(),
            &self
                .username
                .chars()
                .map(|c| format!("{} ", c))
                .collect::<String>(),
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
        // 硬核阴影绘制
        for (x, y) in OFFSET {
            draw_text_mut(
                &mut shougou_img,
                Rgba([50, 50, 50, 255]),
                12 + x,
                6 + y,
                Scale::uniform(14.0),
                &font,
                &play_count_info,
            );
        }
        draw_text_mut(
            &mut shougou_img,
            Rgba([255, 255, 255, 255]),
            12,
            6,
            Scale::uniform(14.0),
            &font,
            &play_count_info,
        );

        shougou_img = Self::resize_pic(&shougou_img, 1.05);
        overlay(&mut self.img, &shougou_img, 240, 83);

        // 最核心的 B50 绘制
        self.img = self.draw_best_list()?;

        // 右上角的 Generated By
        let mut author_board_img = image::open(self.pic_dir.join("UI_CMN_MiniDialog_01.png"))?;
        author_board_img = Self::resize_pic(&author_board_img, 0.35);
        draw_text_mut(
            &mut author_board_img,
            Rgba([75, 75, 75, 255]),
            31,
            28,
            Scale::uniform(15.0),
            &font,
            "Generated By",
        );
        draw_text_mut(
            &mut author_board_img,
            Rgba([75, 75, 75, 255]),
            31,
            50,
            Scale::uniform(15.0),
            &font,
            "Maimai-Search",
        );
        overlay(&mut self.img, &author_board_img, 1224, 19);

        // 新歌标签
        overlay(
            &mut self.img,
            &image::open(self.pic_dir.join("UI_RSL_MBase_Parts_01.png"))?,
            988,
            65,
        );

        // 标准标签
        overlay(
            &mut self.img,
            &image::open(self.pic_dir.join("UI_RSL_MBase_Parts_02.png"))?,
            865,
            65,
        );

        let path = LAUNCH_PATH.join("b50.png");
        self.img.save_with_format(&path, ImageFormat::Png)?;
        open::that(&path).unwrap();
        Ok(())
    }
}
