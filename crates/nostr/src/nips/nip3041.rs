use alloc::string::String;
use alloc::vec::Vec;


use crate::{EventId, Tag};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollData {
    /// <multi|single> allow others to reply with one or multiple options
    pub multi_select: String,
    /// LC(in depth|VLC) when surv expires,0 LC the uplimit of VLC(2M)
    pub clock: String,
    /// event name
    pub title: String,
    /// optional Poll & Vote information ("" if not present)
    pub info: String,
    /// array of string
    pub options: Vec<String>,
}

impl PollData {
    /// Create a new poll
    pub fn new(multi_select: bool, clock: &str, title: &str, info: &str, options: &Vec<String>) -> Self {
        let mut ms = "single";
        if multi_select {
            ms = "multi"
        }
        Self {
            multi_select: ms.into(),
            clock: clock.into(),
            title: title.into(),
            info: info.into(),
            options: options.to_vec(),
        }
    }
}

impl From<PollData> for Vec<Tag> {
    fn from(value: PollData) -> Self {
        let mut tags = Vec::new();
        tags.push(Tag::Poll {
            multi_select: value.multi_select,
            clock: value.clock,
            title: value.title,
            info: value.info,
            options: value.options,
        });
        tags
    }
}

impl From<PollData> for String {
    fn from(value: PollData) -> Self {
        serde_json::to_string(&value).unwrap_or_default()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteData {
    
    pub event_id :EventId,
    /// array of index of the option | the first option is 0
    pub choices: Vec<String>,
    pub reason_for_voting: String,
}

impl VoteData {
    /// Create a new vote
    pub fn new(ev_id:EventId,choices: &Vec<String>, reason: String) -> Self {
        Self {
            event_id: ev_id,
            choices: choices.to_vec(),
            reason_for_voting: reason,
        }
    }
}

impl From<VoteData> for Vec<Tag> {
    fn from(value: VoteData) -> Self {
        let mut tags = Vec::new();
        tags.push(Tag::Event {
            event_id: value.event_id,
            relay_url: None,
            marker: None,
        });
        tags.push(Tag::Vote {
            choices: value.choices
        });
        tags
    }
}