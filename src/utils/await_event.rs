use std::fmt::Debug;

use serde::Serialize;

use crate::Message;

#[derive(Debug, Default)]
pub struct ExpectedMessages {
	pub changed: bool,
	pub msg_id: Vec<usize>,
	pub responses: Vec<usize>
}

fn await_responses<P: Debug + Serialize>(messages: Vec<Message<P>>) {

}