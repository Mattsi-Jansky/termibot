
#[derive(Debug, PartialEq)]
pub struct CommandData {
    /// The parsed command (first word after bot mention)
    pub command: String,
    /// Arguments passed to the command (everything after the command)
    pub args: Vec<String>,
    /// The raw argument string (unparsed)
    pub raw_args: String,
    /// The channel where the command was issued
    pub channel: String,
    /// The user who issued the command
    pub user: String,
}

#[derive(Debug, PartialEq)]
pub enum EnrichedEvent {
    Command(CommandData),
}
