use crate::CrackProgress;

pub trait Sender: Clone + Send {
    fn send(&self, progress: CrackProgress) -> bool;
}

impl Sender for std::sync::mpsc::Sender<CrackProgress> {
    fn send(&self, progress: CrackProgress) -> bool {
        self.send(progress).is_ok()
    }
}

#[cfg(feature = "tokio")]
impl Sender for tokio::sync::mpsc::Sender<CrackProgress> {
    fn send(&self, progress: CrackProgress) -> bool {
        self.blocking_send(progress).is_ok()
    }
}