use crate::tab::bedrock::block_entry::{Block, BlockMessage};
use crate::tab::controls::{ApplicationTab, CrackerEvent, CrackerState, TabMessage};

use async_std::fs::File;
use async_std::task::spawn_blocking;
use iced::futures::io::BufWriter;
use iced::futures::{AsyncWriteExt, SinkExt};
use iced::{futures, Element, Length};
use iced_native::widget::{column, text};
use iced_native::{subscription, Padding, Subscription};
use bedrock_cracker::{
    estimate_result_amount, search_bedrock_pattern, Block as BlockInfo, CrackProgress,
};

use iced::alignment::Horizontal;
use iced::widget::{Column, Scrollable};
use std::collections::HashSet;
use tokio::sync::mpsc::channel;

#[derive(Debug, Default)]
pub struct BdrkTab {
    estimated_seeds: u64,
    blocks: Vec<Block>,
    valid_blocks: Vec<BlockInfo>,
}

#[derive(Debug, Clone)]
pub enum BdrkMessage {
    Block(usize, BlockMessage),
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
        }
    }

    fn view(&self) -> Element<TabMessage> {
        let estimate = text(format!(
            "Naively estimated results: {} seeds",
            self.estimated_seeds
        ))
        .width(Length::Fill)
        .horizontal_alignment(Horizontal::Center);
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
        let view: Element<_> = Column::with_children(vec![estimate.into(), coords.into()]).into();
        view.map(TabMessage::BdrkMessage)
    }

    fn poll_cracker(&self, state: &CrackerState, threads: &str) -> Subscription<CrackerEvent> {
        match &state {
            CrackerState::Idle => Subscription::none(),
            CrackerState::Starting(file_output) => {
                let threads = threads.parse::<u64>().unwrap_or(1);

                crack(&self.valid_blocks, file_output, threads)
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
        self.update_duplicates();
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

    fn update_duplicates(&mut self) {
        let mut valid_blocks = HashSet::new();
        for block in self.blocks.iter_mut() {
            let is_duplicate = if let Some(pos) = block.get_pos() {
                !valid_blocks.insert(pos)
            } else {
                false
            };
            block.set_duplicate(is_duplicate);
        }
        self.valid_blocks = valid_blocks.into_iter().collect();
    }
}

struct Unique;

pub fn crack(
    blocks: &Vec<BlockInfo>,
    file_output: &Option<String>,
    threads: u64,
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

            spawn_blocking(move || search_bedrock_pattern(&blocks, threads, sender));

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
