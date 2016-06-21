//! `colorparse::parse` parses a color configuration string (in Git syntax)
//! into an `ansi_term::Style`:
//!
//! # Examples
//!
//!     if let Ok(color) = colorparse::parse("bold red blue") {
//!         println!("{}", color.paint("Bold red on blue"));
//!     }
//!
//!     let hyperlink_style = colorparse::parse("#0000ee ul").unwrap();

#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

extern crate ansi_term;
use ansi_term::{Color, Style};

#[macro_use]
extern crate quick_error;

quick_error! {
    /// Type for errors returned by the parser.
    #[derive(Debug, PartialEq)]
    pub enum Error {
        /// An extra color appeared after the foreground and background colors.
        ExtraColor(s: String, word: String) {
            display("Error parsing style \"{}\": extra color \"{}\"", s, word)
        }
        /// An unknown word appeared.
        UnknownWord(s: String, word: String) {
            display("Error parsing style \"{}\": unknown word: \"{}\"", s, word)
        }
    }
}

fn parse_color(word: &str) -> Result<Option<Color>, ()> {
    let color = match word {
        "normal" => None,
        "-1" => None,
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Purple),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        _ => {
            if word.starts_with('#') && word.len() == 7 {
                if let (Ok(r), Ok(g), Ok(b)) = (u8::from_str_radix(&word[1..3], 16),
                                                u8::from_str_radix(&word[3..5], 16),
                                                u8::from_str_radix(&word[5..7], 16)) {
                    return Ok(Some(Color::RGB(r, g, b)))
                }
            } else if let Ok(n) = u8::from_str_radix(word, 10) {
                return Ok(Some(Color::Fixed(n)));
            }
            return Err(());
        }
    };
    Ok(color)
}

/// Parse a string in Git's color configuration syntax into an
/// `ansi_term::Style`.
pub fn parse(s: &str) -> Result<Style, Error> {
    let mut style = Style::new();
    let mut colors = 0;
    let mut bold = false;
    let mut dim = false;
    let mut ul = false;
    let mut blink = false;
    let mut reverse = false;
    for word in s.split_whitespace() {
        match word.to_lowercase().as_ref() {
            "nobold" => { bold = false; }
            "bold" => { bold = true; }
            "nodim" => { dim = false; }
            "dim" => { dim = true; }
            "noul" => { ul = false; }
            "ul" => { ul = true; }
            "noblink" => { blink = false; }
            "blink" => { blink = true; }
            "noreverse" => { reverse = false; }
            "reverse" => { reverse = true; }
            w => {
                if let Ok(color) = parse_color(w) {
                    if colors == 2 {
                        return Err(Error::ExtraColor(s.to_string(), word.to_string()));
                    } else if let Some(color) = color {
                        if colors == 0 {
                            style = style.fg(color);
                        } else if colors == 1 {
                            style = style.on(color);
                        }
                    }
                    colors += 1;
                } else {
                    return Err(Error::UnknownWord(s.to_string(), word.to_string()));
                }
            }
        }
    }
    if bold { style = style.bold(); }
    if dim { style = style.dimmed(); }
    if ul { style = style.underline(); }
    if blink { style = style.blink(); }
    if reverse { style = style.reverse(); }
    Ok(style)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Error::*;
    use ansi_term::Color::*;
    use ansi_term::Style;

    #[test]
    fn test_parse_style() {
        macro_rules! test {
            ($s:expr => $style:expr) => {
                assert_eq!(parse($s), Ok($style));
            };
        }

        test!("" => Style::new());
        test!("  " => Style::new());
        test!("normal" => Style::new());
        test!("normal normal" => Style::new());
        test!("-1 normal" => Style::new());
        test!("red" => Red.normal());
        test!("red blue" => Red.on(Blue));
        test!("   red blue   " => Red.on(Blue));
        test!("red\tblue" => Red.on(Blue));
        test!("red\n blue" => Red.on(Blue));
        test!("red\r\n blue" => Red.on(Blue));
        test!("blue red" => Blue.on(Red));
        test!("yellow green" => Yellow.on(Green));
        test!("white magenta" => White.on(Purple));
        test!("black cyan" => Black.on(Cyan));
        test!("red normal" => Red.normal());
        test!("normal red" => Style::new().on(Red));
        test!("0" => Fixed(0).normal());
        test!("8 3" => Fixed(8).on(Fixed(3)));
        test!("255" => Fixed(255).normal());
        test!("255 -1" => Fixed(255).normal());
        test!("#000000" => RGB(0,0,0).normal());
        test!("#204060" => RGB(0x20,0x40,0x60).normal());

        test!("bold cyan white" => Cyan.on(White).bold());
        test!("bold cyan nobold white" => Cyan.on(White));
        test!("bold cyan reverse white nobold" => Cyan.on(White).reverse());
        test!("bold cyan ul white dim" => Cyan.on(White).bold().underline().dimmed());
        test!("blink #050505 white" => RGB(5,5,5).on(White).blink());
    }

    #[test]
    fn test_parse_style_err() {
        macro_rules! test {
            ($s:expr => $err:ident $word:expr) => {
                assert_eq!(parse($s), Err($err($s.to_string(), $word.to_string())));
            };
        }

        test!("red blue green" => ExtraColor "green");
        test!("red blue 123" => ExtraColor "123");
        test!("123 red blue" => ExtraColor "blue");
        test!("red blue normal" => ExtraColor "normal");
        test!("red blue -1" => ExtraColor "-1");
        test!("yellow green #abcdef" => ExtraColor "#abcdef");
        test!("#123456 #654321 #abcdef" => ExtraColor "#abcdef");
        test!("bold red blue green" => ExtraColor "green");
        test!("red bold blue green" => ExtraColor "green");
        test!("red blue bold green" => ExtraColor "green");
        test!("red blue green bold" => ExtraColor "green");

        test!("256" => UnknownWord "256");
        test!("-2" => UnknownWord "-2");
        test!("-" => UnknownWord "-");
        test!("- 1" => UnknownWord "-");
        test!("123-1" => UnknownWord "123-1");
        test!("blue1" => UnknownWord "blue1");
        test!("blue-1" => UnknownWord "blue-1");
        test!("#" => UnknownWord "#");
        test!("#12345" => UnknownWord "#12345");
        test!("#1234567" => UnknownWord "#1234567");
        test!("#bcdefg" => UnknownWord "#bcdefg");
        test!("#blue" => UnknownWord "#blue");
        test!("blue#123456" => UnknownWord "blue#123456");
    }
}
