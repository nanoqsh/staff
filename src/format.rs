use {
    quick_xml::{events::BytesStart, Error as XmlError, Reader},
    std::{
        borrow::Cow,
        fmt,
        str::{self, FromStr, Utf8Error},
        string::FromUtf8Error,
    },
};

pub(crate) fn read<'a>(src: &'a str) -> Result<Document, Failed> {
    use {
        quick_xml::{events::Event, name::QName},
        std::mem,
    };

    let mut library_geometries = false;
    let mut doc = Document { geometry: vec![] };
    let mut sources = vec![];
    let mut indxs = vec![];
    let mut inputs = vec![];

    let mut reader = Reader::from_str(src);
    let mut stack = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name() {
                QName(b"library_geometries") => library_geometries = true,
                QName(b"geometry") if library_geometries => {
                    stack.push(El::Geometry {
                        id: e.get_attribute_as_string("id").into_failed(&reader)?,
                        name: e.get_attribute_as_string("name").into_failed(&reader)?,
                    });
                }
                QName(b"source") if library_geometries => {
                    stack.push(El::Source {
                        id: e.get_attribute_as_string("id").into_failed(&reader)?,
                    });
                }
                QName(b"float_array") if library_geometries => {
                    let count = e.get_attribute_as_parsed("count").into_failed(&reader)?;
                    let floats = Vec::with_capacity(count);
                    stack.push(El::FloatArray { floats });
                }
                QName(b"triangles") if library_geometries => {
                    let count = e.get_attribute_as_parsed("count").into_failed(&reader)?;
                    let indxs = Vec::with_capacity(count);
                    stack.push(El::Triangles { indxs });
                }
                _ => {}
            },
            Ok(Event::End(e)) => match e.name() {
                QName(b"library_geometries") => library_geometries = false,
                QName(b"geometry") if library_geometries => {
                    let Some(El::Geometry { id, name }) = stack.pop() else {
                        return Err(Failed::new(Error::UnexpectedClosingTag("geometry".to_string()), &reader));
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
                QName(b"source") if library_geometries => {
                    let Some(El::Source { id }) = stack.pop() else {
                        return Err(Failed::new(Error::UnexpectedClosingTag("source".to_string()), &reader));
                    };

                    if let Some(source) = sources.last_mut() {
                        source.id = id;
                    }
                }
                QName(b"float_array") if library_geometries => {
                    let Some(El::FloatArray { floats }) = stack.pop() else {
                        return Err(Failed::new(Error::UnexpectedClosingTag("float_array".to_string()), &reader));
                    };

                    sources.push(Source {
                        id: String::new(),
                        floats,
                    });
                }
                QName(b"triangles") if library_geometries => {
                    let Some(El::Triangles { indxs: i }) = stack.pop() else {
                        return Err(Failed::new(Error::UnexpectedClosingTag("triangles".to_string()), &reader));
                    };

                    indxs = i;
                }
                _ => {}
            },
            Ok(Event::Empty(e)) => {
                let Some(El::Triangles { .. }) = stack.last() else {
                    continue;
                };

                inputs.push(Input {
                    source: e.get_attribute_as_string("source").into_failed(&reader)?,
                    offset: e.get_attribute_as_parsed("offset").into_failed(&reader)?,
                });
            }
            Ok(Event::Text(e)) => match stack.last_mut() {
                Some(El::FloatArray { floats, .. }) => {
                    let e = str::from_utf8(&e)
                        .map_err(Into::into)
                        .into_failed(&reader)?;

                    for f in e.split_whitespace() {
                        let f = f
                            .parse()
                            .map_err(|_| Error::Parse(f.to_owned()))
                            .into_failed(&reader)?;

                        floats.push(f);
                    }
                }
                Some(El::Triangles { indxs }) => {
                    let e = str::from_utf8(&e)
                        .map_err(Into::into)
                        .into_failed(&reader)?;

                    for i in e.split_whitespace() {
                        let i = i
                            .parse()
                            .map_err(|_| Error::Parse(i.to_owned()))
                            .into_failed(&reader)?;

                        indxs.push(i);
                    }
                }
                _ => (),
            },
            Ok(Event::Eof) => break,
            Err(err) => return Err(Failed::new(Error::XmlError(err), &reader)),
            _ => {}
        }
    }

    Ok(doc)
}

pub(crate) struct Failed {
    err: Error,
    pos: usize,
}

impl Failed {
    fn new<R>(err: Error, reader: &Reader<R>) -> Self {
        Self {
            err,
            pos: reader.buffer_position(),
        }
    }
}

impl fmt::Display for Failed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} at position {}", self.err, self.pos)
    }
}

pub(crate) enum Error {
    UnexpectedClosingTag(String),
    AttributeNotFound(String),
    Parse(String),
    Utf8Error(Utf8Error),
    FromUtf8Error(FromUtf8Error),
    XmlError(XmlError),
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
            Self::UnexpectedClosingTag(tag) => write!(f, "unexpected closing tag {tag}"),
            Self::AttributeNotFound(attr) => write!(f, "the attribute {attr} not found"),
            Self::Parse(s) => write!(f, "failed to parse {s:?} string"),
            Self::Utf8Error(err) => write!(f, "{err}"),
            Self::FromUtf8Error(err) => write!(f, "{err}"),
            Self::XmlError(err) => write!(f, "{err}"),
        }
    }
}

trait IntoFailed<T> {
    fn into_failed<R>(self, reader: &Reader<R>) -> Result<T, Failed>;
}

impl<T> IntoFailed<T> for Result<T, Error> {
    fn into_failed<R>(self, reader: &Reader<R>) -> Result<T, Failed> {
        self.map_err(|err| Failed::new(err, reader))
    }
}

#[derive(Debug)]
pub(crate) struct Document {
    pub geometry: Vec<Geometry>,
}

#[derive(Debug)]
pub(crate) struct Geometry {
    pub id: String,
    pub name: String,
    pub sources: Vec<Source>,
    pub triangles: Triangles,
}

#[derive(Debug)]
pub(crate) struct Triangles {
    pub indxs: Vec<u32>,
    pub inputs: Vec<Input>,
}

#[derive(Debug)]
pub(crate) struct Input {
    pub source: String,
    pub offset: usize,
}

#[derive(Debug)]
pub(crate) struct Source {
    pub id: String,
    pub floats: Vec<f32>,
}

enum El {
    Geometry { id: String, name: String },
    Source { id: String },
    FloatArray { floats: Vec<f32> },
    Triangles { indxs: Vec<u32> },
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
        Ok(String::from_utf8(at.to_vec())?)
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
