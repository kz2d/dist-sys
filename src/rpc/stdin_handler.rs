use std::{thread, io, sync::mpsc::Sender};
use std::fmt::Debug;
use serde::{Serialize, Deserialize};

use crate::Message;

pub fn message_handler<'a, P: Deserialize<'a> + Debug + Clone + Serialize + Send + 'static>(sn: Sender<Message<P>>, chain: &[Box<dyn Fn(String) -> bool>]) {
    thread::spawn(move || {
        let stdin = io::stdin();
        for i in stdin.lines() {
            let i = i.unwrap();
			for z in chain {
				if z(i) {
					continue;
				}
			}
			let i: Message::<P> = serde_json::from_str(i.as_str()).unwrap();
            eprintln!("in: {:?}", i);

            sn.send(i.clone()).unwrap();
        }
    });
}
