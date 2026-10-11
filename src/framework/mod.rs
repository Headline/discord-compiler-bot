use std::collections::{HashMap, HashSet};
use std::time::Duration;

use futures_util::future::BoxFuture;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::id::UserId;

use crate::events;
use ratelimit::RateLimiter;

mod ratelimit;

pub type CommandError = Box<dyn std::error::Error + Send + Sync>;
pub type CommandResult<T = ()> = Result<T, CommandError>;

/// Signature of a prefix command, receiving the text that follows the command name
pub type CommandFn = for<'a> fn(&'a Context, &'a Message, &'a str) -> BoxFuture<'a, CommandResult>;

/// A prefix command and the conditions under which it may run
pub struct Command {
    /// Primary name followed by any aliases
    pub names: &'static [&'static str],
    pub owners_only: bool,
    pub rate_limited: bool,
    pub run: CommandFn,
}

/// Routes prefixed messages to their commands
pub struct Dispatcher {
    prefix: String,
    owners: HashSet<UserId>,
    commands: HashMap<&'static str, &'static Command>,
    rate_limiter: RateLimiter,
}

impl Dispatcher {
    pub fn new(prefix: &str, owners: HashSet<UserId>, commands: &'static [Command]) -> Self {
        let mut map = HashMap::new();
        for command in commands {
            for name in command.names {
                map.insert(*name, command);
            }
        }

        Dispatcher {
            prefix: prefix.to_owned(),
            owners,
            commands: map,
            // at most 3 commands every 10 seconds, each at least 3 seconds apart
            rate_limiter: RateLimiter::new(Duration::from_secs(3), Duration::from_secs(10), 3),
        }
    }

    /// Runs the command contained in the message, if any
    pub async fn dispatch(&self, ctx: &Context, msg: &Message) {
        if msg.author.bot || msg.webhook_id.is_some() {
            return;
        }

        let Some((name, args)) = self.parse(&msg.content) else {
            return;
        };
        let Some(command) = self.commands.get(name) else {
            return;
        };

        let is_owner = self.owners.contains(&msg.author.id);
        if command.owners_only && !is_owner {
            return;
        }

        if command.rate_limited && !is_owner && !self.rate_limiter.take(msg.author.id) {
            events::ratelimited(ctx, msg).await;
            return;
        }

        if !events::before(ctx, msg).await {
            return;
        }

        let result = (command.run)(ctx, msg, args).await;
        events::after(ctx, msg, result).await;
    }

    /// Splits a message into its command name and the remaining arguments
    fn parse<'a>(&self, content: &'a str) -> Option<(&'a str, &'a str)> {
        let rest = content.trim_start().strip_prefix(self.prefix.as_str())?;
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let (name, args) = rest.split_at(end);
        Some((name, args.trim_start()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dispatcher() -> Dispatcher {
        Dispatcher::new(";", HashSet::new(), &[])
    }

    #[test]
    fn parses_name_and_args() {
        let d = dispatcher();
        assert_eq!(d.parse(";compile c++ -O2"), Some(("compile", "c++ -O2")));
        assert_eq!(d.parse("  ;ping"), Some(("ping", "")));
        assert_eq!(
            d.parse(";compile c++\n```cpp\nint main() {}\n```"),
            Some(("compile", "c++\n```cpp\nint main() {}\n```"))
        );
    }

    #[test]
    fn rejects_missing_prefix() {
        let d = dispatcher();
        assert_eq!(d.parse("compile c++"), None);
        assert_eq!(d.parse("hello ;compile"), None);
    }

    #[test]
    fn requires_name_directly_after_prefix() {
        let d = dispatcher();
        assert_eq!(d.parse("; compile"), Some(("", "compile")));
        assert_eq!(d.parse(";compile```cpp"), Some(("compile```cpp", "")));
    }
}
