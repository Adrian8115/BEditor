use std::fs;

use bedrock_rs::core::read::ByteStreamRead;
use bedrock_rs::nbt::big_endian::NbtBigEndian;
use bedrock_rs::nbt::little_endian::NbtLittleEndian;
use bedrock_rs::nbt::little_endian_network::NbtLittleEndianNetwork;
use bedrock_rs::nbt::NbtTag;
use iced::widget::{Column, Row, Scrollable, Text, TextInput};
use iced::{Element, Length, Padding, Sandbox};

use crate::messages::BEditorMessage;
use crate::view::BEditorView;

pub const INDENTATION: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NbtEndian {
    #[default]
    Little,
    LittleNetwork,
    Big,
}

impl NbtEndian {
    const ALL: [NbtEndian; 3] = [NbtEndian::Little, NbtEndian::LittleNetwork, NbtEndian::Big];
}

impl std::fmt::Display for NbtEndian {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NbtEndian::Little => "Little Endian",
                NbtEndian::LittleNetwork => "Little Endian Network",
                NbtEndian::Big => "Big Endian",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NbtHeader {
    #[default]
    None,
    Normal,
    LevelDat,
}

impl NbtHeader {
    const ALL: [NbtHeader; 3] = [NbtHeader::None, NbtHeader::Normal, NbtHeader::LevelDat];
}

impl std::fmt::Display for NbtHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NbtHeader::None => "No Header",
                NbtHeader::Normal => "Normal Header",
                NbtHeader::LevelDat => "Level.dat Header",
            }
        )
    }
}

pub struct NbtView {
    path: String,
    nbt: Result<(String, NbtTag, Option<(i32, i32)>), String>,
    endian: NbtEndian,
    header: NbtHeader,
}

impl NbtView {
    fn parse_nbt(&self) -> Result<(String, NbtTag, Option<(i32, i32)>), String> {
        let data = match fs::read(self.path.clone()) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("Error reading File: {e:?}"));
            }
        };

        let mut stream = ByteStreamRead::from(data);

        let mut header = None;

        match self.header {
            NbtHeader::None => {}
            NbtHeader::Normal | NbtHeader::LevelDat => {
                let first = match stream.read_i32le() {
                    Ok(v) => v.0,
                    Err(e) => {
                        return Err(format!("Error reading Nbt header: {e:?}"));
                    }
                };

                let second = match stream.read_i32le() {
                    Ok(v) => v.0,
                    Err(e) => {
                        return Err(format!("Error reading Nbt header: {e:?}"));
                    }
                };

                header = Some((first, second))
            }
        }

        match self.endian {
            NbtEndian::Little => match NbtTag::nbt_deserialize::<NbtLittleEndian>(&mut stream) {
                Ok(v) => Ok((v.0, v.1, header)),
                Err(e) => Err(format!("Error parsing Nbt: {e:?}")),
            },
            NbtEndian::LittleNetwork => {
                match NbtTag::nbt_deserialize::<NbtLittleEndianNetwork>(&mut stream) {
                    Ok(v) => Ok((v.0, v.1, header)),
                    Err(e) => Err(format!("Error parsing Nbt: {e:?}")),
                }
            }
            NbtEndian::Big => match NbtTag::nbt_deserialize::<NbtBigEndian>(&mut stream) {
                Ok(v) => Ok((v.0, v.1, header)),
                Err(e) => Err(format!("Error parsing Nbt: {e:?}")),
            },
        }
    }

    fn nbt2elements(&self, name: String, tag: NbtTag, indent: u32) -> Element<BEditorMessage> {
        let padding = Padding {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: indent as f32 * INDENTATION,
        };

        match tag {
            NbtTag::Byte(v) => Column::new()
                .push(Text::new(format!(
                    "{name}{}Byte({v})",
                    if !name.is_empty() { ": " } else { "" }
                )))
                .padding(padding)
                .into(),
            NbtTag::Int16(v) => Column::new()
                .push(Text::new(format!(
                    "{name}{}Int16({v})",
                    if !name.is_empty() { ": " } else { "" }
                )))
                .padding(padding)
                .into(),
            NbtTag::Int32(v) => Column::new()
                .push(Text::new(format!(
                    "{name}{}Int32({v})",
                    if !name.is_empty() { ": " } else { "" }
                )))
                .padding(padding)
                .into(),
            NbtTag::Int64(v) => Column::new()
                .push(Text::new(format!(
                    "{name}{}Int64({v})",
                    if !name.is_empty() { ": " } else { "" }
                )))
                .padding(padding)
                .into(),
            NbtTag::Float32(v) => Column::new()
                .push(Text::new(format!(
                    "{name}{}Float32({v})",
                    if !name.is_empty() { ": " } else { "" }
                )))
                .padding(padding)
                .into(),
            NbtTag::Float64(v) => Column::new()
                .push(Text::new(format!(
                    "{name}{}Float64({v})",
                    if !name.is_empty() { ": " } else { "" }
                )))
                .padding(padding)
                .into(),
            NbtTag::String(v) => Column::new()
                .push(Text::new(format!(
                    "{name}{}\"{v}\"",
                    if !name.is_empty() { ": " } else { "" }
                )))
                .padding(padding)
                .into(),
            NbtTag::List(v) => {
                let col = Column::new();

                let mut col = col.push(Text::new(format!(
                    "{name}{}[",
                    if !name.is_empty() { ": " } else { "" }
                )));

                for nbt in v.iter() {
                    col = col.push(self.nbt2elements("".to_string(), nbt.clone(), indent + 1));
                }

                col = col.push(Text::new(String::from("]")));

                col.padding(padding).into()
            }
            NbtTag::Compound(v) => {
                let mut col = Column::new();

                col = col.push(Text::new(format!(
                    "{name}{}{{",
                    if !name.is_empty() { ": " } else { "" }
                )));

                for (str, nbt) in v.iter() {
                    col = col.push(self.nbt2elements(str.clone(), nbt.clone(), indent + 1));
                }

                col = col.push(Text::new(format!("}}")));

                col.padding(padding).into()
            }
            NbtTag::Empty => Column::new()
                .push(Text::new(format!("{name}: EMPTY")))
                .padding(padding)
                .into(),
        }
    }
}

impl BEditorView for NbtView {
    fn new() -> Self {
        Self {
            path: String::new(),
            nbt: Err(String::from("")),
            endian: Default::default(),
            header: NbtHeader::None,
        }
    }

    fn update(&mut self, message: BEditorMessage) {
        match message {
            BEditorMessage::NbtViewSetPath(v) => self.path = v,
            BEditorMessage::NbtViewSetEndian(v) => self.endian = v,
            BEditorMessage::NbtViewSetHeader(v) => self.header = v,
            BEditorMessage::NbtViewRefresh => {}
        }

        self.nbt = self.parse_nbt();
    }

    fn view(&self) -> Element<BEditorMessage> {
        let padding = Padding {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: INDENTATION,
        };

        Column::new()
            .push(
                Row::new()
                    .push(
                        TextInput::new("Your Path", &self.path)
                            .on_input(BEditorMessage::NbtViewSetPath),
                    )
                    .push(iced::widget::PickList::new(
                        &NbtEndian::ALL[..],
                        Some(self.endian),
                        |s| BEditorMessage::NbtViewSetEndian(s),
                    ))
                    .push(iced::widget::PickList::new(
                        &NbtHeader::ALL[..],
                        Some(self.header),
                        |s| BEditorMessage::NbtViewSetHeader(s),
                    ))
                    .push(
                        iced::widget::Button::new(Text::new("Refresh"))
                            .on_press(BEditorMessage::NbtViewRefresh),
                    ),
            )
            .push(
                Scrollable::new(match &self.nbt {
                    Ok(v) => {
                        let col = Column::new();

                        let col = match v.clone().2 {
                            None => col,
                            Some(v) => match self.header {
                                NbtHeader::None => col,
                                NbtHeader::Normal => {
                                    let col = col.push(Text::new(String::from("Header: {")));

                                    let col2 = Column::new();
                                    let col2 = col2.push(Text::new(format!("First: {}", v.0)));
                                    let col2 = col2.push(Text::new(format!("Length: {}", v.1)));

                                    let col = col.push(col2.padding(padding));

                                    col.push(Text::new(String::from("}")))
                                }
                                NbtHeader::LevelDat => {
                                    let col = col.push(Text::new(String::from("Header: {")));

                                    let col2 = Column::new();
                                    let col2 =
                                        col2.push(Text::new(format!("Format Version: {}", v.0)));
                                    let col2 = col2.push(Text::new(format!("Length: {}", v.1)));

                                    let col = col.push(col2.padding(padding));

                                    col.push(Text::new(String::from("}")))
                                }
                            },
                        };

                        col.push(self.nbt2elements(v.clone().0, v.clone().1, 1))
                    }
                    Err(e) => Column::new().push(Text::new(format!("{e}"))),
                })
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .into()
    }
}
