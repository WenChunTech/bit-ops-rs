use iced::font::{Family, Weight};
use iced::widget::{Column, Container, Row, button, text, text_input};
use iced::{Color, Element, Font, Renderer, Task, Theme};
use iced::{Length, Size};

use bitvec::prelude::*;

pub fn main() -> iced::Result {
    iced::application("位操作器", Bit64::update, Bit64::view)
        .font(include_bytes!("../fonts/SourceHanSansCN-Normal.otf").as_slice())
        .default_font(Font {
            family: Family::Name("思源黑体"),
            weight: Weight::Normal,
            ..Default::default()
        })
        .default_font(Font::with_name("黑体-简"))
        .window_size(Size::new(1080.0, 410.0)) // 调整窗口大小
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
        }
    }
}

#[derive(Debug, Clone)]
enum BitOps {
    Toggled(u8, u8),
    ShiftLeft(u32),            // 修改为包含位移数
    ShiftRight(u32),           // 修改为包含位移数
    ExpressionChanged(String), // 新增：处理输入框变化
    HexChanged(String),        // 新增
    DecChanged(String),        // 新增
    OctChanged(String),        // 新增
    BinChanged(String),        // 新增
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
            .push(make_data_row("数据大小:", "Size", &self.size, |_| None));

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
                println!("十六进制输入值：{}", value);
                match parse_number(&value, 16) {
                    Some(num) => {
                        self.data = num;
                        self.update_displays();
                        Task::none()
                    }
                    None => {
                        self.update_displays_default();
                        return Task::none();
                    }
                }
            }
            BitOps::DecChanged(value) => {
                match parse_number(&value, 10) {
                    Some(num) => {
                        self.data = num;
                        self.update_displays();
                    }
                    None => {
                        self.update_displays_default();
                        return Task::none();
                    }
                }
                Task::none()
            }
            BitOps::OctChanged(value) => match parse_number(&value, 8) {
                Some(num) => {
                    self.data = num;
                    Task::none()
                }
                None => {
                    self.update_displays_default();
                    return Task::none();
                }
            },
            BitOps::BinChanged(value) => match parse_number(&value, 2) {
                Some(num) => {
                    self.data = num;
                    self.update_displays();
                    Task::none()
                }
                None => {
                    self.update_displays_default();
                    return Task::none();
                }
            },
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
                .on_input(move |s| on_change(s.clone()).unwrap_or(BitOps::ExpressionChanged(s))),
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
        .push(text(index.to_string()).center().color(txt_color))
        .push(
            button(text(bit).width(Length::Fill).center().color(txt_color))
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
    use crate::data_size;

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
