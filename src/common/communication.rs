
use std::sync::mpsc::{Sender, Receiver, channel};
use super::color::UITile;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum KeyAction {
    Right,
    Left,
    Up,
    Down,
    Wait,
    Exit,
    None,
}

pub struct Communicator<T1, T2> {
    s: Sender<T1>,
    r: Receiver<T2>,
}

impl<T1,T2> Communicator<T1,T2> {
    pub fn pair() -> (Communicator<T1, T2>, Communicator<T2, T1>) {
        let (in1, out1): (Sender<T1>, Receiver<T1>) = channel();
        let (in2, out2): (Sender<T2>, Receiver<T2>) = channel();
        return (Communicator {s: in1, r: out2}, Communicator {s: in2, r: out1});
    }
    
    pub fn send(&self, t: T1) {
        self.s.send(t).unwrap();
    }
    
    pub fn recv(&self) -> T2 {
        return self.r.recv().unwrap();
    }
}

pub enum GameMessage {
    TileWin(usize, usize, Vec<UITile>),
    WaitingForKey,
    // Signals that this game has exited.
    Exit,
    None,
}

pub enum UIMessage {
    KeyAction(KeyAction),
    // Signals that the game SHOULD EXIT; the UI keeps going!
    Exit,
    None,
}
