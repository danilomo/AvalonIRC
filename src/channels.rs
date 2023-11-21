use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::collections::HashSet;

static EMPTY_SET: Lazy<HashSet<String>> = Lazy::new(|| HashSet::new());

pub struct Channels {
    channels_map: HashMap<String, HashSet<String>>,
}

impl Channels {
    pub fn new() -> Self {
        Channels {
            channels_map: HashMap::new(),
        }
    }

    pub fn join_user(&mut self, channel: &str, nick: &str) {
        let channel_map = self.channels_map.get_mut(channel);

        match channel_map {
            Some(chan) => {
                chan.insert(nick.into());
            }
            None => {
                let mut set = HashSet::new();
                set.insert(nick.into());
                self.channels_map.insert(channel.into(), set);
            }
        }
    }

    pub fn channel_list(&self, channel: &str) -> impl Iterator<Item = &String> {
        if let Some(chan) = self.channels_map.get(channel) {
            return chan.into_iter();
        }

        EMPTY_SET.iter()
    }
}
