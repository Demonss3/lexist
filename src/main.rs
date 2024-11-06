use std::{collections::HashSet, fs::read_to_string, path::Path, str::from_utf8};

use genanki_rs::{Deck, Field, Model, Note, Template};
use lexist::tokenizer::SfTokenizer;
use mdict::MDictBuilder;
use quick_xml::{events::Event, Reader};
use sudachi::prelude::Mode;

use epub::doc::EpubDoc;

#[derive(Debug)]
struct NoteType {
    front: String,
    back: String,
}

fn main() {
    
}

fn lookup_word(word: &str) -> String {
    let buider = MDictBuilder::new("resources/mdx/Shogakukanjcv3.mdx");

    let mut dict = buider.build().unwrap();

    match dict.lookup(word).unwrap() {
        Some(res) => {
            let defination = res.definition;
            if defination.starts_with("@") {
                println!("{}", defination);
                let key = defination.split("=").collect::<Vec<&str>>()[1].trim();
                println!("{}", key);
                match dict.lookup(key).unwrap() {
                    Some(res) => {
                        return res.definition;
                    }
                    None => {
                        return String::new();
                    } 
                }
            } else {
                return defination;
            }
        }
        None => {
            return String::new();
        }
    }
}

fn run() {

    let txts = read_txts_from_epubs(Path::new("resources/epub/1.epub"));

    let mut tokenizer = SfTokenizer::new_built(Mode::C);

    let mut words = HashSet::new();

    txts.iter().for_each(|txt| {
        let res = tokenizer.tokenize(txt);

        res.iter().for_each(|morph| {
            words.insert(morph.dictionary_form().to_string());
        });
    });

    let mut notfound = Vec::new();
    let mut notes = Vec::new();

    let buider = MDictBuilder::new("resources/mdx/Shogakukanjcv3.mdx");

    let mut dict = buider.build().unwrap();
    words.iter().for_each(|word| {

        match dict.lookup(word).unwrap() {
            Some(res) => {
                let mut note: NoteType = NoteType {
                    front: word.clone(),
                    back: String::new(),
                };
                let defination = res.definition;
                if defination.starts_with("@") {
                    let key = defination.split("=").collect::<Vec<&str>>()[1].trim();
                    dict.lookup(key).unwrap().map(|res| {
                        note.back = res.definition;
                    });
                } else {

                    note.back = defination;
                }

                notes.push(note);
            }
            None => {
                notfound.push(word);
            }
            
        }
    });

    notfound.iter().for_each(|key| {
        println!("{}", key);
    });

    let css = Path::new("resources/css/Shogakukanjcv3.css");
    let css = read_to_string(css).unwrap();

    let model = Model::new(
        1607392319,
        "Simple Model",
        vec![
            Field::new("Front"),
            Field::new("Back"),
        ],
        vec![Template::new("Card 1").qfmt("{{Front}}").afmt("{{FrontSide}}<hr id=answer>{{Back}}")],
    ).css(css);
    let mut deck = Deck::new(1234, "Example Deck", "Example Deck");
    notes.iter().for_each(|note| {
        deck.add_note(Note::new(model.clone(), vec![&note.front, &note.back]).unwrap());
    });
    deck.write_to_file("output.apkg").unwrap();
}

fn tokenize_txt(text: &str) -> Vec<String> {
    let mut tokenizer = SfTokenizer::new_built(Mode::C);
    let res = tokenizer.tokenize(text);

    res.iter()
        .map(|morph| morph.surface().to_string())
        .collect()
}

fn xml_inside_tags(content: &str, tag: &str) -> HashSet<String> {
    let mut inside_tag = false;

    let mut rdr = Reader::from_str(content);

    let mut ret = HashSet::new();

    loop {
        match rdr.read_event() {
            Err(e) => panic!("Error at position {}: {:?}", rdr.error_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                _ if tag.as_bytes() == e.name().as_ref() => {
                    inside_tag = true;
                }
                other => {
                    if inside_tag {
                        ret.insert(from_utf8(other).unwrap().to_string());
                    }
                }
            },
            Ok(Event::End(e)) => {
                if e.name().as_ref() == tag.as_bytes() {
                    inside_tag = false;
                }
            }
            _ => (),
        }
    }
    ret
}

fn read_txts_from_epubs(path: &Path) -> Vec<String> {
    let mut doc = EpubDoc::new(path).unwrap();
    let spines = doc.spine.clone();

    let mut txt = Vec::new();

    spines.iter().for_each(|id| {
        let res = doc.get_resource_str(id).unwrap();
        let content = res.0;
        let mut rdr = Reader::from_str(&content);
        rdr.config_mut().trim_text(true);

        let mut inside_p_content = String::new();

        let mut inside_p_tag = false;
        let mut inside_ruby_tag = false;
        let mut inside_rt_tag = false;

        loop {
            match rdr.read_event() {
                Err(e) => panic!("Error at position {}: {:?}", rdr.error_position(), e),

                Ok(Event::Eof) => break,

                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"p" => {
                        inside_p_tag = true;
                    }
                    b"ruby" => {
                        inside_ruby_tag = true;
                    }
                    b"rt" => {
                        inside_rt_tag = true;
                    }
                    _ => (),
                },

                Ok(Event::Text(e)) if inside_p_tag => {
                    if inside_p_tag || inside_ruby_tag {
                        inside_p_content.push_str(e.unescape().unwrap().as_ref());
                    } else if inside_rt_tag {
                        inside_p_content
                            .push_str(format!("（{}）", e.unescape().unwrap()).as_str());
                    }
                }

                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"p" => {
                        inside_p_tag = false;
                        txt.push(inside_p_content.clone());
                        inside_p_content.clear();
                    }
                    b"ruby" => {
                        inside_ruby_tag = false;
                    }
                    b"rt" => {
                        inside_rt_tag = false;
                    }
                    _ => (),
                },

                _ => (),
            }
        }
    });

    txt
}
