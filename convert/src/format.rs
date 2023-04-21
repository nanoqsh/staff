use {
    quick_xml::{
        events::{self, BytesStart},
        Error as XmlError, Reader,
    },
    std::{
        borrow::Cow,
        fmt, mem,
        str::{self, FromStr, Utf8Error},
        string::FromUtf8Error,
    },
};

#[derive(Default)]
pub(crate) struct Document {
    pub geometry: Vec<Geometry>,
    pub nodes: Vec<Node>,
    pub animations: Vec<Animation>,
}

pub(crate) struct Geometry {
    pub id: String,
    pub name: String,
    pub sources: Vec<Source>,
    pub triangles: Triangles,
}

pub(crate) struct Triangles {
    pub indxs: Vec<u32>,
    pub inputs: Vec<Input>,
}

pub(crate) struct Input {
    pub source: String,
    pub offset: usize,
}

pub(crate) struct Source {
    pub id: String,
    pub floats: Vec<f32>,
    pub names: Vec<Name>,
}

pub(crate) struct Node {
    pub id: String,
    pub name: String,
    pub ty: String,
    pub mat: Vec<f32>,
    pub children: Vec<Self>,
}

pub(crate) struct Animation {
    pub id: String,
    pub name: String,
    pub sources: Vec<Source>,
}

pub(crate) enum Name {
    Linear,
    Bezier,
}

impl Name {
    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "LINEAR" => Ok(Self::Linear),
            "BEZIER" => Ok(Self::Bezier),
            _ => Err(Error::Name(s.to_owned())),
        }
    }
}

pub(crate) fn read(src: &str) -> Result<Document, Failed> {
    let mut reader = Reader::from_str(src);
    read_from_reader(&mut reader).map_err(|err| {
        let mut pos = reader.buffer_position();
        let mut line = 1;
        for line_len in src.lines().map(str::len) {
            match pos.checked_sub(line_len) {
                Some(p) => pos = p,
                None => break,
            }

            line += 1;
        }

        Failed { err, line }
    })
}

#[allow(clippy::too_many_lines)]
fn read_from_reader(reader: &mut Reader<&[u8]>) -> Result<Document, Error> {
    use events::Event;

    enum Library {
        None,
        Geometries,
        VisualScenes,
        Animations,
    }

    let mut library = Library::None;
    let mut doc = Document::default();

    let mut sources = vec![];
    let mut indxs = vec![];
    let mut inputs = vec![];

    let mut stack = vec![];
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"library_geometries" => library = Library::Geometries,
                b"library_visual_scenes" => library = Library::VisualScenes,
                b"library_animations" => library = Library::Animations,
                b"geometry" => {
                    if let Library::Geometries = library {
                        stack.push(El::Geometry {
                            id: e.get_attribute_as_string("id")?,
                            name: e.get_attribute_as_string("name")?,
                        });
                    }
                }
                b"source" => {
                    if let Library::Geometries | Library::Animations = library {
                        stack.push(El::Source {
                            id: e.get_attribute_as_string("id")?,
                        });
                    }
                }
                b"float_array" => {
                    if let Library::Geometries | Library::Animations = library {
                        let count = e.get_attribute_as_parsed("count")?;
                        let floats = Vec::with_capacity(count);
                        stack.push(El::FloatArray { floats });
                    }
                }
                b"Name_array" => {
                    if let Library::Animations = library {
                        let count = e.get_attribute_as_parsed("count")?;
                        let names = Vec::with_capacity(count);
                        stack.push(El::NameArray { names });
                    }
                }
                b"triangles" => {
                    if let Library::Geometries = library {
                        let count = e.get_attribute_as_parsed("count")?;
                        let indxs = Vec::with_capacity(count);
                        stack.push(El::Triangles { indxs });
                    }
                }
                b"node" => {
                    if let Library::VisualScenes = library {
                        stack.push(El::Node(Node {
                            id: e.get_attribute_as_string("id")?,
                            name: e.get_attribute_as_string("name")?,
                            ty: e.get_attribute_as_string("type")?,
                            mat: vec![],
                            children: vec![],
                        }));
                    }
                }
                b"matrix" => {
                    if let Library::VisualScenes = library {
                        stack.push(El::Mat);
                    }
                }
                b"animation" => {
                    if let Library::Animations = library {
                        stack.push(El::Animation {
                            id: e.get_attribute_as_string("id")?,
                            name: e.get_attribute_as_string("name")?,
                        });
                    }
                }
                _ => {}
            },
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"library_geometries" | b"library_visual_scenes" | b"library_animations" => {
                    library = Library::None;
                }
                b"geometry" => {
                    if let Library::Geometries = library {
                        let Some(El::Geometry { id, name }) = stack.pop() else {
                            return Err(Error::UnexpectedClosingTag("geometry".to_owned()));
                        };

                        doc.geometry.push(Geometry {
                            id,
                            name,
                            sources: mem::take(&mut sources),
                            triangles: Triangles {
                                indxs: mem::take(&mut indxs),
                                inputs: mem::take(&mut inputs),
                            },
                        });
                    }
                }
                b"source" => {
                    if let Library::Geometries | Library::Animations = library {
                        let Some(El::Source { id }) = stack.pop() else {
                            return Err(Error::UnexpectedClosingTag("source".to_owned()));
                        };

                        if let Some(source) = sources.last_mut() {
                            source.id = id;
                        }
                    }
                }
                b"float_array" => {
                    if let Library::Geometries | Library::Animations = library {
                        let Some(El::FloatArray { floats }) = stack.pop() else {
                            return Err(Error::UnexpectedClosingTag("float_array".to_owned()));
                        };

                        sources.push(Source {
                            id: String::new(),
                            floats,
                            names: vec![],
                        });
                    }
                }
                b"Name_array" => {
                    if let Library::Animations = library {
                        let Some(El::NameArray { names }) = stack.pop() else {
                            return Err(Error::UnexpectedClosingTag("Name_array".to_owned()));
                        };

                        sources.push(Source {
                            id: String::new(),
                            floats: vec![],
                            names,
                        });
                    }
                }
                b"triangles" => {
                    if let Library::Geometries = library {
                        let Some(El::Triangles { indxs: i }) = stack.pop() else {
                            return Err(Error::UnexpectedClosingTag("triangles".to_owned()));
                        };

                        indxs = i;
                    }
                }
                b"node" => {
                    if let Library::VisualScenes = library {
                        let Some(El::Node(node)) = stack.pop() else {
                            return Err(Error::UnexpectedClosingTag("node".to_owned()));
                        };

                        if let Some(El::Node(Node { children, .. })) = stack.last_mut() {
                            children.push(node);
                        } else {
                            doc.nodes.push(node);
                        }
                    }
                }
                b"matrix" => {
                    if let Library::VisualScenes = library {
                        let Some(El::Mat) = stack.pop() else {
                            return Err(Error::UnexpectedClosingTag("matrix".to_owned()));
                        };
                    }
                }
                b"animation" => {
                    if let Library::Animations = library {
                        let Some(El::Animation { id, name }) = stack.pop() else {
                            return Err(Error::UnexpectedClosingTag("animation".to_owned()));
                        };

                        doc.animations.push(Animation {
                            id,
                            name,
                            sources: mem::take(&mut sources),
                        });
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(e)) => {
                let Some(El::Triangles { .. }) = stack.last() else {
                    continue;
                };

                inputs.push(Input {
                    source: e.get_attribute_as_string("source")?,
                    offset: e.get_attribute_as_parsed("offset")?,
                });
            }
            Ok(Event::Text(e)) => match stack.last_mut() {
                Some(El::FloatArray { floats, .. }) => {
                    let e = str::from_utf8(&e)?;
                    for f in e.split_whitespace() {
                        let f = f.parse().map_err(|_| Error::Parse(f.to_owned()))?;
                        floats.push(f);
                    }
                }
                Some(El::Triangles { indxs }) => {
                    let e = str::from_utf8(&e)?;
                    for i in e.split_whitespace() {
                        let i = i.parse().map_err(|_| Error::Parse(i.to_owned()))?;
                        indxs.push(i);
                    }
                }
                Some(El::Mat) => {
                    let Some(El::Node(Node { mat, .. })) = stack.iter_mut().rev().nth(1) else {
                        return Err(Error::MatrixNotFound);
                    };

                    let e = str::from_utf8(&e)?;
                    for f in e.split_whitespace() {
                        let f = f.parse().map_err(|_| Error::Parse(f.to_owned()))?;
                        mat.push(f);
                    }
                }
                Some(El::NameArray { names }) => {
                    let e = str::from_utf8(&e)?;
                    for n in e.split_whitespace() {
                        names.push(Name::from_str(n)?);
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(err) => return Err(Error::XmlError(err)),
            _ => {}
        }
    }

    Ok(doc)
}

pub struct Failed {
    pub err: Error,
    pub line: usize,
}

impl fmt::Display for Failed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} at line {}", self.err, self.line)
    }
}

pub enum Error {
    UnexpectedClosingTag(String),
    MatrixNotFound,
    AttributeNotFound(String),
    Parse(String),
    Utf8Error(Utf8Error),
    FromUtf8Error(FromUtf8Error),
    XmlError(XmlError),
    Name(String),
}

impl From<Utf8Error> for Error {
    fn from(v: Utf8Error) -> Self {
        Self::Utf8Error(v)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(v: FromUtf8Error) -> Self {
        Self::FromUtf8Error(v)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedClosingTag(tag) => write!(f, "unexpected closing tag {tag:?}"),
            Self::MatrixNotFound => write!(f, "matrix not found"),
            Self::AttributeNotFound(attr) => write!(f, "the attribute {attr:?} not found"),
            Self::Parse(s) => write!(f, "failed to parse {s:?} string"),
            Self::Utf8Error(err) => write!(f, "{err}"),
            Self::FromUtf8Error(err) => write!(f, "{err}"),
            Self::XmlError(err) => write!(f, "{err}"),
            Self::Name(name) => write!(f, "unknown name {name:?}"),
        }
    }
}

enum El {
    Geometry { id: String, name: String },
    Source { id: String },
    FloatArray { floats: Vec<f32> },
    Triangles { indxs: Vec<u32> },
    Node(Node),
    Mat,
    Animation { id: String, name: String },
    NameArray { names: Vec<Name> },
}

trait GetAttribute {
    fn get_attribute<'a>(&'a self, attr: &str) -> Result<Cow<'a, [u8]>, Error>;

    fn get_attribute_as_parsed<T>(&self, attr: &str) -> Result<T, Error>
    where
        T: FromStr,
    {
        let at = self.get_attribute(attr)?;
        let s = str::from_utf8(&at)?;
        s.parse().map_err(|_| Error::Parse(s.to_owned()))
    }

    fn get_attribute_as_string(&self, attr: &str) -> Result<String, Error> {
        let at = self.get_attribute(attr)?;
        Ok(String::from_utf8(at.into_owned())?)
    }
}

impl GetAttribute for BytesStart<'_> {
    fn get_attribute<'a>(&'a self, attr: &str) -> Result<Cow<'a, [u8]>, Error> {
        self.try_get_attribute(attr.as_bytes())
            .ok()
            .flatten()
            .map(|at| at.value)
            .ok_or_else(|| Error::AttributeNotFound(attr.to_owned()))
    }
}
