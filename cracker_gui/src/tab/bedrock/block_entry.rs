use crate::tab::input_error_style::TextInputErrorStyle;
use iced::{widget, Element, Renderer, Theme};
use iced_native::row;
use iced_native::theme::TextInput;
use iced_native::widget::{button, pick_list, text_input};
use bedrock_cracker::{Block as BlockInfo, BlockType};

#[derive(Debug, Clone)]
pub struct Block {
    x: Coord,
    y: Coord,
    z: Coord,
    block_type: BlockType,
    duplicate: bool,
}

#[derive(Debug, Clone)]
struct Coord {
    text: String,
    num: Option<i32>,
    coord_type: CoordType,
}

impl Coord {
    fn new(coord_type: CoordType) -> Self {
        Coord {
            text: "".to_string(),
            num: None,
            coord_type,
        }
    }
}

#[derive(Debug, Clone)]
enum CoordType {
    XZ,
    Y,
}

impl CoordType {
    fn check_valid(&self, coord: i32) -> bool {
        match self {
            CoordType::XZ => (-30000000..=30000000).contains(&coord),
            CoordType::Y => (1..=4).contains(&coord) || (123..=126).contains(&coord),
        }
    }
}

impl Coord {
    fn update(&mut self, text: String) {
        self.num = match text.parse::<i32>() {
            Ok(num) if self.coord_type.check_valid(num) => Some(num),
            _ => None,
        };
        self.text = text;
    }

    fn view<'a, F>(
        &self,
        last: bool,
        duplicate: bool,
        callback: F,
    ) -> widget::TextInput<'a, BlockMessage, Renderer<Theme>>
    where
        F: 'a + Fn(String) -> BlockMessage,
    {
        let mut coord = text_input("", &self.text).on_input(callback);

        if !last {
            if duplicate {
                coord = coord.style(TextInput::Custom(Box::new(TextInputErrorStyle)));
            } else if self.num.is_none() {
                coord = coord.style(TextInput::Custom(Box::new(TextInputErrorStyle)));
            }
        }

        coord
    }
}

#[derive(Debug, Clone)]
pub enum BlockMessage {
    EditedX(String),
    EditedY(String),
    EditedZ(String),
    EditedType(BlockType),
    Deleted,
}

impl Block {
    pub fn new() -> Self {
        Self {
            x: Coord::new(CoordType::XZ),
            y: Coord::new(CoordType::Y),
            z: Coord::new(CoordType::XZ),
            block_type: BlockType::BEDROCK,
            duplicate: false,
        }
    }

    pub fn update(&mut self, message: BlockMessage) {
        match message {
            BlockMessage::EditedX(x) => self.x.update(x),
            BlockMessage::EditedY(y) => self.y.update(y),
            BlockMessage::EditedZ(z) => self.z.update(z),
            BlockMessage::Deleted => {
                unreachable!()
            }
            BlockMessage::EditedType(block_type) => {
                self.block_type = block_type;
            }
        }
    }

    pub fn view(&self, last: bool) -> Element<BlockMessage> {
        let x = self.x.view(last, self.duplicate, BlockMessage::EditedX);
        let y = self.y.view(last, self.duplicate, BlockMessage::EditedY);
        let z = self.z.view(last, self.duplicate, BlockMessage::EditedZ);
        let selection = pick_list(
            &BlockType::ALL[..],
            Some(self.block_type),
            BlockMessage::EditedType,
        );
        let mut delete = button("Delete");

        if !last {
            delete = delete.on_press(BlockMessage::Deleted);
        }

        row![x, y, z, selection, delete].spacing(5).into()
    }

    pub fn is_empty(&self) -> bool {
        &self.x.text == "" && self.y.text == "" && self.z.text == ""
    }

    pub fn get_pos(&self) -> Option<BlockInfo> {
        match (self.x.num, self.y.num, self.z.num) {
            (Some(x), Some(y), Some(z)) => Some(BlockInfo::new(x, y, z, self.block_type.clone())),
            _ => None,
        }
    }

    pub fn set_duplicate(&mut self, duplicate: bool) {
        self.duplicate = duplicate;
    }
}

impl From<&str> for Block {
    fn from(line: &str) -> Self {
        let mut block = Block::new();
        let mut components = line.split_whitespace();
        block
            .x
            .update(components.next().unwrap_or_default().to_string());
        block
            .y
            .update(components.next().unwrap_or_default().to_string());
        block
            .z
            .update(components.next().unwrap_or_default().to_string());
        if components.next().unwrap_or_default().to_ascii_lowercase()
            == format!("{}", BlockType::OTHER)
        {
            block.block_type = BlockType::OTHER;
        }
        block
    }
}
