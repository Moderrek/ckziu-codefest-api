use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize_repr, Serialize_repr, )]
#[non_exhaustive]
#[repr(u8)]
pub enum Opcode {
  Identify = 0,
  Ready = 1,
  Heartbeat = 2,
}
