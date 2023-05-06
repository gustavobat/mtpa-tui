pub enum Tab {
    Encrypted,
    Decryption,
}

pub enum InputMode {
    Normal,
    Editing,
}

pub struct App<'a> {
    pub titles: Vec<&'a str>,
    pub current_tab: Tab,
    pub input: String,
    pub input_mode: InputMode,
    pub encrypted_messages: Vec<Vec<u8>>,
    pub decrypted_messages: Vec<Vec<u8>>,
    pub key: Vec<Option<u8>>,
    pub position: (usize, usize),
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            titles: vec!["Encrypted", "Decryption"],
            current_tab: Tab::Encrypted,
            input: String::new(),
            input_mode: InputMode::Normal,
            encrypted_messages: Vec::new(),
            decrypted_messages: Vec::new(),
            key: Vec::new(),
            position: (0, 0),
        }
    }

    pub fn toggle_tab(&mut self) {
        match self.current_tab {
            Tab::Encrypted => self.current_tab = Tab::Decryption,
            Tab::Decryption => self.current_tab = Tab::Encrypted,
        }
        self.position = (0, 0);
    }

    pub fn get_current_tab_index(&self) -> usize {
        match self.current_tab {
            Tab::Encrypted => 0,
            Tab::Decryption => 1,
        }
    }
}
