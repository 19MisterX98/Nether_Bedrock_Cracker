use crate::CrackProgress;

pub trait Sender: Clone + Send {
    fn send(&self, progress: CrackProgress) -> bool;
}

impl Sender for std::sync::mpsc::Sender<CrackProgress> {
    fn send(&self, progress: CrackProgress) -> bool {
        self.send(progress).is_ok()
    }
}