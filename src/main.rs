#![windows_subsystem = "windows"]

use iced::alignment::Horizontal;
use iced::font::{Family, Weight};
use iced::widget::{Column, Container, Row, button, text, text_input};
use iced::{Color, Element, Font, Renderer, Task, Theme};
use iced::{Length, Size};

use bitvec::prelude::*;

pub fn main() -> iced::Result {
    iced::application("位操作器", Bit64::update, Bit64::view)
        .font(include_bytes!("../fonts/LXGWWenKai-Regular.ttf").as_slice())
        .default_font(Font {
            family: Family::Name("霞鹜文楷"),
            weight: Weight::Normal,
            ..Default::default()
        })
        .window_size(Size::new(1080.0, 410.0))
        .run()
}

#[derive(Debug)]
struct Bit64 {
    data: u64,
    expression: String,
    hex: String,
    dec: String,
    oct: String,
    bin: String,
    size: String,
    size_input: String, // 新增：存储正在输入的数据大小
}

impl Default for Bit64 {
    fn default() -> Self {
        Self {
            data: 0,
            expression: String::new(),
            hex: String::new(),
            dec: String::new(),
            oct: String::new(),
            bin: String::new(),
            size: String::from("0 B"),
            size_input: String::from("0 B"),
        }
    }
}

#[derive(Debug, Clone)]
enum BitOps {
    None,
    Toggled(u8, u8),
    ShiftLeft(u32),            // 修改为包含位移数
    ShiftRight(u32),           // 修改为包含位移数
    ExpressionChanged(String), // 新增：处理输入框变化
    HexChanged(String),        // 新增
    DecChanged(String),        // 新增
    OctChanged(String),        // 新增
    BinChanged(String),        // 新增
    DataSizeChanged(String),   // 新增
}

impl Bit64 {
    fn view(&self) -> Element<BitOps> {
        let bits = self.data.view_bits::<Lsb0>();
        // 从下到上显示：先收集成Vec，然后反转
        let mut chunks: Vec<_> = bits.chunks(16).enumerate().collect();
        chunks.reverse();

        let pane = chunks
            .iter()
            .fold(Column::new().spacing(20), |col, (row_idx, chunk)| {
                // 从右到左显示：收集并反转每一行的位
                let mut bits_vec: Vec<_> = chunk.iter().enumerate().collect();
                bits_vec.reverse();

                let row = bits_vec
                    .iter()
                    .fold(Row::new().spacing(10), |row, (col_idx, bit)| {
                        let bit = if **bit { "1" } else { "0" };
                        // 保持行号不变（因为已经反转过），列号使用反转后的索引
                        row.push(item(bit, *row_idx, *col_idx))
                    });

                col.push(row)
            });

        let shift_controls = Row::new()
            .spacing(10)
            .push(
                button(text("<<").width(Length::Fill))
                    .on_press(BitOps::ShiftLeft(self.parse_shift_amount())),
            )
            .push(
                text_input("移位数", &self.expression)
                    .on_input(BitOps::ExpressionChanged)
                    .padding(5),
            )
            .push(
                button(text(">>").width(Length::Fill))
                    .on_press(BitOps::ShiftRight(self.parse_shift_amount())),
            );

        let left = Column::new()
            .spacing(20)
            .padding(5)
            .push(pane)
            .push(shift_controls); // 使用新的控制行替换原来的 text_input

        let right = Column::new()
            .spacing(33.5)
            .padding(5)
            .push(make_data_row("十六进制:", "Hex", &self.hex, |s| {
                Some(BitOps::HexChanged(s))
            }))
            .push(make_data_row("十进制:", "Sep", &self.dec, |s| {
                Some(BitOps::DecChanged(s))
            }))
            .push(make_data_row("八进制:", "Oct", &self.oct, |s| {
                Some(BitOps::OctChanged(s))
            }))
            .push(make_data_row("二进制:", "Bin", &self.bin, |s| {
                Some(BitOps::BinChanged(s))
            }))
            .push(make_data_row(
                "数据大小:",
                "Size",
                &self.size_input,
                |s| Some(BitOps::DataSizeChanged(s)),
            ));

        let content = Row::new()
            .spacing(5)
            .padding(20) // 增加整体内边距
            .push(left)
            .push(right);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn update(&mut self, message: BitOps) -> Task<BitOps> {
        match message {
            BitOps::None => Task::none(),
            BitOps::Toggled(i, j) => {
                // 计算实际的位索引：
                // i 是从下到上的行号（0-3）
                // j 是从右到左的列号（0-15）
                let index = i as usize * 16 + j as usize;
                let bits_mut = self.data.view_bits_mut::<Lsb0>();
                bits_mut.set(index, !bits_mut[index]);

                // 更新各种进制的显示
                self.update_displays();
                Task::none()
            }
            BitOps::ShiftLeft(amount) => {
                self.data = self.data.checked_shl(amount).unwrap_or(0);
                self.update_displays();
                Task::none()
            }
            BitOps::ShiftRight(amount) => {
                self.data = self.data.checked_shr(amount).unwrap_or(0);
                self.update_displays();
                Task::none()
            }
            BitOps::ExpressionChanged(value) => {
                self.expression = value;
                Task::none()
            }
            BitOps::HexChanged(value) => {
                match parse_number(&value, 16) {
                    Some(num) => {
                        self.data = num;
                        self.update_displays();
                    }
                    None => {
                        self.update_displays_default();
                    }
                }
                Task::none()
            }
            BitOps::DecChanged(value) => {
                match parse_number(&value, 10) {
                    Some(num) => {
                        self.data = num;
                        self.update_displays();
                    }
                    None => {
                        self.update_displays_default();
                    }
                }
                Task::none()
            }
            BitOps::OctChanged(value) => {
                match parse_number(&value, 8) {
                    Some(num) => {
                        self.data = num;
                        self.update_displays();
                    }
                    None => {
                        self.update_displays_default();
                    }
                }
                Task::none()
            }
            BitOps::BinChanged(value) => {
                match parse_number(&value, 2) {
                    Some(num) => {
                        self.data = num;
                        self.update_displays();
                    }
                    None => {
                        self.update_displays_default();
                    }
                }
                Task::none()
            }
            BitOps::DataSizeChanged(value) => {
                self.size_input = value.clone();
                if value.is_empty() {
                    self.update_displays_default();
                } else if let Some(bytes) = parse_data_size(&value) {
                    self.data = bytes;
                    self.update_displays();
                    self.size_input = self.size.clone(); // 更新为格式化后的显示
                }
                Task::none()
            }
        }
    }

    // 新增辅助方法，用于更新所有显示
    fn update_displays(&mut self) {
        // 更新所有显示
        self.hex = format!("{:X}", self.data);
        self.dec = format!("{}", self.data);
        self.oct = format!("{:o}", self.data);
        self.bin = format!("{:b}", self.data);
        self.size = data_size(self.data);
        self.size_input = self.size.clone(); // 同步 size_input
    }

    fn update_displays_default(&mut self) {
        self.data = 0;
        self.hex = String::default();
        self.dec = String::default();
        self.oct = String::default();
        self.bin = String::default();
        self.size = "0 B".to_string();
    }

    // 新增：解析位移数量的辅助方法
    fn parse_shift_amount(&self) -> u32 {
        self.expression
            .parse()
            .unwrap_or(1) // 如果解析失败，默认移动1位
            .min(64) // 限制最大移动位数为64
    }
}

fn parse_data_size(s: &str) -> Option<u64> {
    if s.is_empty() {
        return None;
    }

    let mut total: u64 = 0;
    let s = s.to_uppercase();
    let mut chars = s.chars();
    let mut num_buffer = String::new();

    let mut prev_was_unit = true; // 确保开始时是数字

    while let Some(c) = chars.next() {
        if c.is_ascii_whitespace() {
            continue;
        }

        if c.is_ascii_digit() {
            if !prev_was_unit {
                return None; // 单位后面必须是空格或数字
            }
            num_buffer.push(c);
            continue;
        }

        // 处理单位字符
        if matches!(c, 'B' | 'K' | 'M' | 'G' | 'T') {
            if num_buffer.is_empty() {
                return None; // 单位前必须有数字
            }

            let num: u64 = num_buffer.parse().ok()?;
            let bytes = match c {
                'B' => num,
                'K' => num.checked_mul(1024)?,
                'M' => num.checked_mul(1024 * 1024)?,
                'G' => num.checked_mul(1024 * 1024 * 1024)?,
                'T' => num.checked_mul(1024 * 1024 * 1024 * 1024)?,
                _ => return None,
            };

            total = total.checked_add(bytes)?;
            num_buffer.clear();
            prev_was_unit = true;
            continue;
        }

        return None; // 遇到非法字符
    }

    if !num_buffer.is_empty() {
        return None; // 有未处理完的数字
    }

    Some(total)
}

// 改进数字解析函数
fn parse_number(s: &str, radix: u32) -> Option<u64> {
    if s.is_empty() {
        return None;
    }

    let s = s.trim().to_uppercase();
    // 移除可能的前缀
    let s = match radix {
        16 => s.trim_start_matches("0X"),
        2 => s.trim_start_matches("0B"),
        8 => s.trim_start_matches("0O"),
        _ => &s,
    };

    // 验证字符是否合法
    if !s.chars().all(|c| c.is_digit(radix)) {
        return None;
    }

    // 尝试解析数字
    Some(u64::from_str_radix(s, radix).unwrap_or(0))
}

// 修改 make_data_row 函数的闭包处理
fn make_data_row<'a, F>(
    label: &'a str,
    placeholder: &str,
    value: &str,
    on_change: F,
) -> Row<'a, BitOps>
where
    F: 'static + Fn(String) -> Option<BitOps>,
{
    Row::new()
        .spacing(10)
        .align_y(iced::alignment::Alignment::Center)
        .push(
            text(label)
                .width(Length::Fixed(100.))
                .align_x(iced::alignment::Horizontal::Right)
                .size(20),
        )
        .push(
            text_input(placeholder, value)
                .padding(10)
                .width(Length::Fixed(300.))
                .on_input(move |s| on_change(s.clone()).unwrap_or(BitOps::None)),
        )
}

fn item(bit: &str, i: usize, j: usize) -> Column<'_, BitOps, Theme, Renderer> {
    // 计算实际的位索引，用于显示和操作
    let index = i * 16 + j;
    let txt_color = if j / 4 % 2 == 0 {
        Color::from_rgb8(230, 28, 139)
    } else {
        Color::from_rgb8(21, 151, 30)
    };

    Column::new()
        .spacing(5)
        .align_x(Horizontal::Center)
        .push(text(index.to_string()).center().color(txt_color))
        .push(
            button(text(bit).width(Length::Fill).center())
                .style(move |_theme, _status| {
                    if bit == "1" {
                        button::Style {
                            background: Some(iced::Background::Color(Color::from_rgb8(
                                229, 22, 22, //rgb(43, 187, 227)
                            ))),
                            ..Default::default()
                        }
                    } else {
                        button::Style {
                            background: Some(iced::Background::Color(Color::from_rgb8(
                                43, 187, 227,
                            ))),
                            ..Default::default()
                        }
                    }
                })
                .on_press(BitOps::Toggled(i as u8, j as u8)),
        )
}

fn data_size(data: u64) -> String {
    if data == 0 {
        return "0 B".to_string();
    }

    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    let mut result = String::new();
    let mut remaining = data;

    if remaining >= TB {
        let t = remaining / TB;
        remaining %= TB;
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&format!("{}T", t));
    }

    if remaining >= GB {
        let g = remaining / GB;
        remaining %= GB;
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&format!("{}G", g));
    }

    if remaining >= MB {
        let m = remaining / MB;
        remaining %= MB;
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&format!("{}M", m));
    }

    if remaining >= KB {
        let k = remaining / KB;
        remaining %= KB;
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&format!("{}K", k));
    }

    if remaining > 0 || result.is_empty() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&format!("{}B", remaining));
    }

    result
}

#[cfg(test)]
mod test {
    use crate::{data_size, parse_data_size};

    #[test]
    fn test_parse_data_size() {
        // 基本格式测试
        assert_eq!(parse_data_size("7B"), Some(7));
        assert_eq!(parse_data_size("1K"), Some(1024));
        assert_eq!(parse_data_size("2K"), Some(2048));

        // 空格分隔的格式
        assert_eq!(parse_data_size("1K 512B"), Some(1536));
        assert_eq!(parse_data_size("1M 12B"), Some(1024 * 1024 + 12));
        assert_eq!(
            parse_data_size("1G 1K 1B"),
            Some(1024 * 1024 * 1024 + 1024 + 1)
        );

        // 无空格分隔的格式
        assert_eq!(parse_data_size("1K1B"), Some(1025));
        assert_eq!(parse_data_size("1M1K"), Some(1024 * 1024 + 1024));
        assert_eq!(
            parse_data_size("1G1M1K1B"),
            Some(1024 * 1024 * 1024 + 1024 * 1024 + 1024 + 1)
        );

        // 错误情况
        assert_eq!(parse_data_size(""), None);
        assert_eq!(parse_data_size("invalid"), None);
        assert_eq!(parse_data_size("123X"), None); // 无效单位
        assert_eq!(parse_data_size("1KB"), None); // 单位之间缺少数字
    }

    #[test]
    fn test_data_size() {
        assert_eq!(data_size(7), "7B");
        assert_eq!(data_size(1024), "1K");
        assert_eq!(data_size(2048), "2K");
        assert_eq!(data_size(1536), "1K 512B");
        assert_eq!(data_size(1024 * 1024), "1M");
        assert_eq!(data_size(1024 * 1024 + 12), "1M 12B");
        assert_eq!(data_size(1024 * 1024 * 1024 + 1024 + 1), "1G 1K 1B");
    }
}
