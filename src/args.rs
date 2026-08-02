const SAVE_INPUT: &str = "--save-input";

pub struct Arguments {
    pub save_input: bool,
}

impl Arguments {
    pub fn parse(arguments: impl IntoIterator<Item = String>) -> Arguments {
        let save_input = arguments
            .into_iter()
            .skip(1)
            .any(|argument| argument == SAVE_INPUT);

        Arguments { save_input }
    }
}
