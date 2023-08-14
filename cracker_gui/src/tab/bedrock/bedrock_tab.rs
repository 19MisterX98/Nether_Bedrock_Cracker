use crate::tab::bedrock::block_entry::{Block, BlockMessage};
use crate::tab::controls::{ApplicationTab, CrackerEvent, CrackerState, TabMessage};

use async_std::fs::File;
use async_std::task::spawn_blocking;
use iced::futures::io::BufWriter;
use iced::futures::{AsyncWriteExt, SinkExt};
use iced::{futures, Element, Length};
use iced_native::widget::{column, pick_list, text};
use iced_native::{subscription, Padding, Subscription, row};
use bedrock_cracker::{CrackProgress, estimate_result_amount, search_bedrock_pattern};
use bedrock_cracker::raw_data::block::Block as BlockInfo;

use iced::widget::{Column, Scrollable};
use tokio::sync::mpsc::channel;
use bedrock_cracker::raw_data::block_type::BlockType;
use bedrock_cracker::raw_data::modes::{CrackerMode, OutputMode};

#[derive(Debug, Default)]
pub struct BdrkTab {
    estimated_seeds: u64,
    blocks: Vec<Block>,
    valid_blocks: Vec<BlockInfo>,
    mode: CrackerMode,
    output_mode: OutputMode,
}

#[derive(Debug, Clone)]
pub enum BdrkMessage {
    Block(usize, BlockMessage),
    CrackerMode(CrackerMode),
    OutputMode(OutputMode),
}

impl From<TabMessage> for BdrkMessage {
    fn from(tab_msg: TabMessage) -> Self {
        match tab_msg {
            TabMessage::BdrkMessage(msg) => msg,
        }
    }
}

impl ApplicationTab for BdrkTab {
    type Message = BdrkMessage;

    fn new() -> Self {
        Self {
            estimated_seeds: (1 << 48),
            blocks: vec![Block::new()],
            valid_blocks: Vec::new(),
            mode: CrackerMode::Normal,
            output_mode: OutputMode::WorldSeed,
        }
    }

    fn load_config(&mut self, file: String) {
        let mut blocks = vec![];
        for line in file.lines().take(200) {
            let block = Block::from(line);
            blocks.push(block);
        }
        self.blocks = blocks;
        self.update_blocks();
    }

    fn save_config(&self) -> String {
        let mut content = String::new();
        for block in self.valid_blocks.iter() {
            content.push_str(&format!("{}\n", block))
        }
        content
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            BdrkMessage::Block(index, BlockMessage::Deleted) => {
                self.blocks.remove(index);
                self.update_blocks();
            }

            BdrkMessage::Block(index, message) => {
                if let Some(block) = self.blocks.get_mut(index) {
                    block.update(message);
                    self.update_blocks();
                }
            }
            BdrkMessage::CrackerMode(mode) => {
                self.mode = mode;
                self.update_blocks()
            }
            BdrkMessage::OutputMode(mode) => {
                self.output_mode = mode;
            }
        }
    }

    fn view(&self) -> Element<TabMessage> {
        let estimate = text(format!(
            "Naively estimated results: {} seeds",
            self.estimated_seeds
        ))
            .width(Length::Fill);
        let crack_mode = pick_list(
            &CrackerMode::ALL[..],
            Some(self.mode),
            BdrkMessage::CrackerMode,
        );
        let output_mode = pick_list(
            &OutputMode::ALL[..],
            Some(self.output_mode),
            BdrkMessage::OutputMode,
        );
        let top_bar = row![estimate, crack_mode, output_mode];
        let coords: Element<_> = column(
            self.blocks
                .iter()
                .enumerate()
                .map(|(i, task)| {
                    task.view(i == self.blocks.len() - 1)
                        .map(move |message| BdrkMessage::Block(i, message))
                })
                .collect(),
        )
        .padding(Padding::from([5, 20]))
        .spacing(5)
        .into();
        let coords = Scrollable::new(coords).height(Length::Fill);
        let view: Element<_> = Column::with_children(vec![top_bar.into(), coords.into()]).into();
        view.map(TabMessage::BdrkMessage)
    }

    fn poll_cracker(&self, state: &CrackerState, threads: &str) -> Subscription<CrackerEvent> {
        match &state {
            CrackerState::Idle => Subscription::none(),
            CrackerState::Starting(file_output) => {
                let threads = threads.parse::<u64>().unwrap_or(1);

                crack(&self.valid_blocks, file_output, threads, self.mode, self.output_mode)
            }
            CrackerState::Running => subscription::run_with_id(
                std::any::TypeId::of::<Unique>(),
                futures::stream::pending(),
            ),
        }
    }
}

impl BdrkTab {
    fn update_blocks(&mut self) {
        self.add_entry();
        self.update_invalid_states();
        self.estimated_seeds = estimate_result_amount(&self.valid_blocks);
    }

    fn add_entry(&mut self) {
        if let Some(last_block) = self.blocks.last() {
            if !last_block.is_empty() {
                self.blocks.push(Block::new())
            }
        } else {
            self.blocks.push(Block::new())
        }
    }

    /// check for multiple blocks in the same position etc...
    fn update_invalid_states(&mut self) {
        let mut valid_blocks: Vec<BlockInfo> = vec![];
        for gui_block in self.blocks.iter_mut() {
            if let Some(block) = gui_block.is_valid_pos() {
                let invalid = Self::check_invalid(&block, &valid_blocks, self.mode);
                if !invalid {
                    valid_blocks.push(block);
                }
                gui_block.set_duplicate(invalid);
            }
        }
        self.valid_blocks = valid_blocks;
    }

    fn check_invalid(block: &BlockInfo, valid_blocks: &[BlockInfo], mode: CrackerMode) -> bool {
        for valid_block in valid_blocks.iter() {
            if block.x == valid_block.x &&
                block.z == valid_block.z &&
                (block.y > 5) == (valid_block.y > 5)
            {
                if block.y == valid_block.y { return true }
                if mode == CrackerMode::Paper1_18 {
                    if block.block_type == valid_block.block_type { return true }
                    let mut y1 = valid_block.y;
                    let mut y2 = block.y;
                    if (y1 > 5) ^ (block.block_type == BlockType::OTHER) {
                        (y1, y2) = (y2, y1);
                    }
                    if y1 <= y2 { return true }
                }
            }
        }
        false
    }
}

struct Unique;

pub fn crack(
    blocks: &Vec<BlockInfo>,
    file_output: &Option<String>,
    threads: u64,
    mode: CrackerMode,
    output_mode: OutputMode,
) -> Subscription<CrackerEvent> {
    let file_output = file_output.clone();
    let blocks: Vec<_> = blocks.clone();

    subscription::channel(std::any::TypeId::of::<Unique>(), 100, move |mut output| {
        let file_output = file_output.clone();
        let blocks = blocks.clone();
        async move {
            let mut writer = create_file_writer(&file_output).await;

            output
                .send(CrackerEvent::Started)
                .await
                .expect("TODO: panic message");

            let (sender, mut receiver) = channel(100);

            spawn_blocking(move || search_bedrock_pattern(&blocks, threads, mode, output_mode, sender));

            let mut seeds = vec![];
            while let Some(pl_event) = receiver.recv().await {
                match pl_event {
                    CrackProgress::Progress(num) => {
                        let percentage = num as f32 / (1u64 << 48) as f32;
                        let update = CrackerEvent::ProgressUpdate(percentage, seeds);
                        seeds = vec![];
                        output.send(update).await.unwrap();
                    }
                    CrackProgress::Seed(num) => {
                        let seed = (num as i64).to_string();
                        if let Some(ref mut writer) = writer {
                            let line = format!("{}\n", seed);
                            writer.write_all(line.as_bytes()).await.unwrap();
                        }
                        seeds.push(seed);
                    }
                };
            }
            output.send(CrackerEvent::Finished).await.unwrap();

            iced::futures::future::pending().await
        }
    })
}

async fn create_file_writer(file: &Option<String>) -> Option<BufWriter<File>> {
    match file {
        None => None,
        Some(file) => {
            let file = File::create(file).await.ok()?;
            Some(BufWriter::new(file))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_invalid() {
        let block = BlockInfo::new(1,1,1,BlockType::BEDROCK);
        let mut valid_blocks = vec![
            BlockInfo::new(1,2,1,BlockType::BEDROCK),
            BlockInfo::new(1,3,1,BlockType::BEDROCK)
        ];
        //no duplicates -> block is valid
        assert!(!BdrkTab::check_invalid(&block, &valid_blocks, CrackerMode::Normal));

        //duplicates -> block is invalid
        valid_blocks.push(block.clone());
        assert!(BdrkTab::check_invalid(&block, &valid_blocks, CrackerMode::Normal));
    }

    #[test]
    fn test_check_invalid_paper() {
        let mut block = BlockInfo::new(1, 1, 1, BlockType::BEDROCK);
        let valid_blocks = vec![
            BlockInfo::new(1,2,1,BlockType::OTHER)
        ];
        //valid position
        assert!(!BdrkTab::check_invalid(&block, &valid_blocks, CrackerMode::Paper1_18));

        //bedrock ont op of other is an invalid placement
        block.y = 3;
        assert!(BdrkTab::check_invalid(&block, &valid_blocks, CrackerMode::Paper1_18));

        //Two of the same type in the same column is redundant
        block.block_type = BlockType::OTHER;
        assert!(BdrkTab::check_invalid(&block, &valid_blocks, CrackerMode::Paper1_18));

        //valid on opposite sites
        block.y = 123;
        assert!(!BdrkTab::check_invalid(&block, &valid_blocks, CrackerMode::Paper1_18));
    }
}
