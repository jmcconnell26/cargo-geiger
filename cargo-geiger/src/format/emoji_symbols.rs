use crate::format::print_config::colorize;
use crate::format::{Charset, CrateDetectionStatus, SymbolKind};

pub struct EmojiSymbols {
    charset: Charset,
    emojis: [&'static str; 3],
    fallbacks: [String; 3],
}

impl EmojiSymbols {
    pub fn emoji(&self, kind: SymbolKind) -> Box<dyn std::fmt::Display> {
        let idx = kind as usize;
        if self.will_output_emoji() {
            Box::new(self.emojis[idx])
        } else {
            Box::new(self.fallbacks[idx].clone())
        }
    }

    pub fn new(charset: Charset) -> EmojiSymbols {
        Self {
            charset,
            emojis: ["🔒", "❓", "☢️"],
            fallbacks: [
                colorize(
                    charset,
                    &CrateDetectionStatus::NoneDetectedForbidsUnsafe,
                    String::from(":)"),
                ),
                colorize(
                    charset,
                    &CrateDetectionStatus::NoneDetectedAllowsUnsafe,
                    String::from("?"),
                ),
                colorize(
                    charset,
                    &CrateDetectionStatus::UnsafeDetected,
                    String::from("!"),
                ),
            ],
        }
    }

    pub fn will_output_emoji(&self) -> bool {
        self.charset == Charset::Utf8
            && console::Term::stdout().features().wants_emoji()
    }
}
