use crate::tab::bedrock::bedrock_tab::{BdrkMessage};

use async_std::fs;
use iced::alignment::Horizontal;
use iced::widget::{Column, Container, Row, Scrollable};
use iced::{Element, Length};
use iced_native::alignment::Vertical;
use iced_native::widget::{button, progress_bar, text, text_input};
use iced_native::{Command, Subscription};
use rfd::AsyncFileDialog;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct ControlMenu<Tab> {
    tab: Tab,
    cracking: CrackerState,
    threads: String,
    crack_data: Option<MetaData>,
}

#[derive(Debug, Clone)]
pub enum ControlMessage {
    TabMessage(TabMessage),
    CrackButton(bool),
    CrackerMessage(CrackerEvent),
    CrackStart(Option<String>),
    ThreadCount(String),
    LoadConfig,
    LoadedConfig(Option<String>),
    SaveConfig,
    None,
}

impl<Tab, Msg> ControlMenu<Tab>
where
    Tab: ApplicationTab<Message = Msg>,
    Msg: From<TabMessage>,
{
    pub fn new() -> ControlMenu<Tab> {
        Self {
            tab: Tab::new(),
            cracking: CrackerState::Idle,
            threads: "".to_string(),
            crack_data: None,
        }
    }

    pub fn view(&self) -> Element<ControlMessage> {
        // coords
        let coord_list = self.tab.view().map(ControlMessage::TabMessage);

        let coord_list: Element<_> = Container::new(coord_list)
            .width(Length::FillPortion(3))
            .into();

        let mut upper_part = Row::new().push(coord_list);

        // results
        if let Some(meta_data) = &self.crack_data {
            if !meta_data.results.is_empty() {
                let mut results = Column::new();

                results = results.push(text("Results:").size(30));

                if meta_data.results.len() == 1000 {
                    results = results.push(
                        text("Not printing more than 1000 results, consider printing to a file")
                            .size(20),
                    );
                }

                for result in meta_data.results.iter() {
                    results =
                        results.push(text_input("", result).on_input(|_| ControlMessage::None));
                }

                let results: Element<_> = Scrollable::new(results)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill)
                    .into();

                upper_part = upper_part.push(results);
            }
        }
        // control panel
        let col = Column::new()
            .push(upper_part.height(Length::Fill))
            .push(self.view_control_panel())
            .height(Length::Fill);

        Container::new(col)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_y(Vertical::Bottom)
            .into()
    }

    pub fn update(&mut self, message: ControlMessage) -> Command<ControlMessage> {
        match message {
            ControlMessage::ThreadCount(text) => self.threads = text,
            ControlMessage::CrackButton(save_to_file) => {
                if self.cracking != CrackerState::Idle {
                    self.end_crack(false)
                } else {
                    if save_to_file {
                        return Command::perform(
                            async move {
                                let file_handle = AsyncFileDialog::new().save_file().await;
                                file_handle
                                    .map(|file| file.path().to_str().map(|str| str.to_string()))
                                    .flatten()
                            },
                            ControlMessage::CrackStart,
                        );
                    } else {
                        self.start_crack(None);
                    }
                }
            }
            ControlMessage::CrackStart(file) => self.start_crack(file),
            ControlMessage::CrackerMessage(msg) => match msg {
                CrackerEvent::ProgressUpdate(num, results) => {
                    let meta_data = self.crack_data.as_mut().unwrap();

                    meta_data.update(num, results.len());

                    let mut results_to_be_added =
                        usize::saturating_sub(1000, meta_data.results.len());
                    results_to_be_added = usize::min(results_to_be_added, results.len());

                    meta_data.results.extend(results.into_iter().take(results_to_be_added));

                    if meta_data.results_found > 10000000 {
                        self.end_crack(true);
                    }
                }
                CrackerEvent::Started => self.cracking = CrackerState::Running,
                CrackerEvent::Finished => self.end_crack(false),
            },
            ControlMessage::LoadConfig => {
                return Command::perform(
                    async {
                        let handle = AsyncFileDialog::new().pick_file().await?;
                        let file = fs::read_to_string(handle.path()).await.ok()?;
                        Some(file)
                    },
                    ControlMessage::LoadedConfig,
                );
            }
            ControlMessage::LoadedConfig(config) => {
                if let Some(config) = config {
                    self.tab.load_config(config);
                }
            }
            ControlMessage::SaveConfig => {
                let config = self.tab.save_config();
                return Command::perform(
                    async move {
                        if let Some(handle) = AsyncFileDialog::new().save_file().await {
                            fs::write(handle.path(), config)
                                .await
                                .expect("Couldnt write contents to file");
                        }
                    },
                    |()| ControlMessage::None,
                );
            }
            ControlMessage::TabMessage(msg) => self.tab.update(msg.into()),
            ControlMessage::None => {}
        }
        Command::none()
    }

    fn view_control_panel(&self) -> Element<ControlMessage> {
        let mut row = Row::new();
        if self.cracking == CrackerState::Idle {
            row = row.push(
                button(text("Crack & save to disk").horizontal_alignment(Horizontal::Center))
                    .on_press(ControlMessage::CrackButton(true)),
            );
            row = row.push(
                button(text("Crack").horizontal_alignment(Horizontal::Center))
                    .on_press(ControlMessage::CrackButton(false)),
            );
        } else {
            row = row.push(
                button("Stop cracking")
                    .on_press(ControlMessage::CrackButton(false))
                    .width(Length::Fixed(130.0)),
            );
        };

        let thread_input = text_input("Threads", &self.threads)
            .on_input(ControlMessage::ThreadCount)
            .width(Length::Fixed(70.0));
        let load_config = button("Load config").on_press(ControlMessage::LoadConfig);
        let save_config = button("Save config").on_press(ControlMessage::SaveConfig);

        row = row.push(thread_input).push(load_config).push(save_config);
        if let Some(meta_data) = &self.crack_data {
            let mut progress_data = Column::new();
            if let TimeElapsed::Running(_, percent) = meta_data.time {
                progress_data = progress_data.push(progress_bar(0.0..=1.0, percent));
            }
            row = row.push(progress_data.push(text(meta_data)));
        }

        row.spacing(5).padding(20).height(Length::Shrink).into()
    }

    pub fn subscription(&self) -> Subscription<ControlMessage> {
        self.tab
            .poll_cracker(&self.cracking, &self.threads)
            .map(ControlMessage::CrackerMessage)
    }

    fn start_crack(&mut self, file: Option<String>) {
        match self.threads.parse::<u64>() {
            Ok(x) if x != 0 => {}
            _ => self.threads = num_cpus::get().to_string(),
        }
        self.crack_data = Some(MetaData::start());
        self.cracking = CrackerState::Starting(file);
    }

    fn end_crack(&mut self, cancelled: bool) {
        self.cracking = CrackerState::Idle;
        self.crack_data.as_mut().unwrap().end(cancelled);
    }
}

#[derive(Debug, Clone)]
pub enum CrackerEvent {
    ProgressUpdate(f32, Vec<String>),
    Started,
    Finished,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub enum CrackerState {
    #[default]
    Idle,
    Starting(Option<String>),
    Running,
}

#[derive(Debug, Clone)]
struct MetaData {
    results_found: usize,
    results: Vec<String>,
    time: TimeElapsed,
}

impl MetaData {
    fn start() -> MetaData {
        MetaData {
            results_found: 0,
            results: vec![],
            time: TimeElapsed::Running(Instant::now(), 0.0),
        }
    }

    fn end(&mut self, cancelled: bool) {
        if cancelled {
            self.time = TimeElapsed::Cancelled
        } else if let TimeElapsed::Running(start, _) = self.time {
            self.time = TimeElapsed::Finished(start.elapsed())
        }
    }

    fn update(&mut self, percent_to_add: f32, results_found: usize) {
        self.results_found += results_found;
        if let TimeElapsed::Running(start, percentage) = self.time {
            self.time = TimeElapsed::Running(start, percentage + percent_to_add);
        }
    }
}

impl fmt::Display for MetaData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Found {} results {}", self.results_found, self.time)
    }
}

#[derive(Debug, Clone)]
enum TimeElapsed {
    Running(Instant, f32),
    Finished(Duration),
    Cancelled,
}

impl fmt::Display for TimeElapsed {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TimeElapsed::Running(instant, percentage) => {
                let elapsed = instant.elapsed().as_secs() as f32;
                let total = elapsed / percentage;
                if total.is_infinite() {
                    write!(f, "in {}s", elapsed)
                } else {
                    write!(f, "in {}s / eta: {}s ", elapsed, total as u64)
                }
            }
            TimeElapsed::Finished(duration) => {
                write!(f, "in {}s", duration.as_secs())
            }
            TimeElapsed::Cancelled => {
                write!(f, "-> cancelled due to too many")
            }
        }
    }
}

pub trait ApplicationTab {
    type Message: From<TabMessage>;

    fn new() -> Self;

    fn load_config(&mut self, file: String);

    fn save_config(&self) -> String;

    fn update(&mut self, message: Self::Message);

    fn view(&self) -> Element<TabMessage>;

    fn poll_cracker(&self, state: &CrackerState, threads: &str) -> Subscription<CrackerEvent>;
}

#[derive(Debug, Clone)]
pub enum TabMessage {
    BdrkMessage(BdrkMessage),
}
