use {
    atlas::{Atlas, Error as AtlasError, ImageData, Map},
    clap::{Parser, Subcommand},
    color::{Color, Error as ColorError},
    convert::{Element, Error as ParseError, Parameters, Target, Value},
    serde_json::Error as JsonError,
    std::{
        env,
        ffi::OsStr,
        fmt,
        fs::{self, File},
        io::{self, BufWriter, Read, Write},
        path::{Path, PathBuf},
        process::ExitCode,
    },
};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Enable verbore output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Convert .dae objects to .json files
    Convert {
        /// Target object to parse (mesh|skeleton|action)
        target: Target,

        /// File to parse (stdin by default)
        filepath: Option<PathBuf>,

        /// Specify output directory (current by default)
        #[arg(short, long)]
        outdir: Option<PathBuf>,
    },
    /// Collect a palette from .png to .json file
    Collect {
        /// File to parse (stdin by default)
        filepath: Option<PathBuf>,

        /// Palette filename ("palette" by default)
        #[arg(short, long)]
        name: Option<String>,

        /// Specify output directory (current by default)
        #[arg(short, long)]
        outdir: Option<PathBuf>,
    },
    /// Repaint .png image with given .json palette
    Repaint {
        /// A path of image to repaint (stdin by default)
        imagepath: Option<PathBuf>,

        /// Palette path (palette.json by default)
        palettepath: Option<PathBuf>,

        /// New image name ("out" by default)
        #[arg(short, long)]
        name: Option<String>,

        /// Specify output directory (current by default)
        #[arg(short, long)]
        outdir: Option<PathBuf>,
    },
    /// Creates a new atlas from given sprite images.
    Atlas {
        /// Pathes of image sprites.
        sprites: Vec<PathBuf>,

        /// The atlas name ("out" by default)
        #[arg(short, long)]
        name: Option<String>,

        /// Specify output directory (current by default)
        #[arg(short, long)]
        outdir: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    if let Err(err) = run(Cli::parse()) {
        eprintln!("error: {err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn run(cli: Cli) -> Result<(), Error> {
    const PALETTE_NAME: &str = "palette";
    const OUT_NAME: &str = "out";

    Parameters {
        verbose: cli.verbose,
        pos_fn: |vs| vs.map(update::<4>),
        map_fn: |[u, v]| [u, 1. - v].map(update::<8>),
        rot_fn: |vs| vs.map(update::<4>),
        act_fn: |vs| vs.map(update::<4>),
        bez_fn: |vs| vs.map(update::<4>),
    }
    .init();

    match cli.command {
        Cmd::Convert {
            target,
            filepath,
            outdir,
        } => {
            let src = read_string(filepath)?;
            let elements = convert::parse(&src, target)?;
            if elements.is_empty() {
                println!("no elements found");
                return Ok(());
            }

            let outdir = make_outdir(outdir)?;
            serialize_elements(&elements, &outdir)
        }
        Cmd::Collect {
            filepath,
            name,
            outdir,
        } => {
            let data = read_data(filepath)?;
            let colors = color::collect(&data)?;
            if colors.is_empty() {
                println!("no colors found");
                return Ok(());
            }

            let name = name.as_deref().unwrap_or(PALETTE_NAME);
            let outdir = make_outdir(outdir)?;
            serialize_colors(&colors, name, &outdir)
        }
        Cmd::Repaint {
            imagepath,
            palettepath,
            name,
            outdir,
        } => {
            let data = read_data(imagepath)?;
            let palette: Vec<Color> = {
                let path = palettepath
                    .or_else(|| {
                        let mut curr = env::current_dir().ok()?;
                        curr.push(PALETTE_NAME);
                        curr.set_extension("json");
                        Some(curr)
                    })
                    .ok_or(Error::PalettePathNotSet)?;

                let src = read_string(Some(path))?;
                serde_json::from_str(&src)?
            };

            let png = color::repaint(&data, &palette)?;
            let name = name.as_deref().unwrap_or(OUT_NAME);
            let outdir = make_outdir(outdir)?;
            write_png(&png, name, &outdir)
        }
        Cmd::Atlas {
            sprites,
            name,
            outdir,
        } => {
            let data = read_sprites(sprites)?;
            let Atlas { png, map } = atlas::atlas(data)?;

            let name = name.as_deref().unwrap_or(OUT_NAME);
            let outdir = make_outdir(outdir)?;
            write_png(&png, name, &outdir)?;
            serialize_map(&map, name, &outdir)
        }
    }
}

fn read_string(path: Option<PathBuf>) -> Result<String, Error> {
    match path {
        Some(path) => fs::read_to_string(&path).map_err(|_| Error::ReadFile(path)),
        None => io::read_to_string(io::stdin()).map_err(|_| Error::ReadStdin),
    }
}

fn read_data(path: Option<PathBuf>) -> Result<Vec<u8>, Error> {
    let stdin_read = || {
        let mut buf = Vec::new();
        io::stdin()
            .read_to_end(&mut buf)
            .map_err(|_| Error::ReadStdin)?;

        Ok(buf)
    };

    match path {
        Some(path) => fs::read(&path).map_err(|_| Error::ReadFile(path)),
        None => stdin_read(),
    }
}

fn read_sprites(sprites: Vec<PathBuf>) -> Result<Vec<ImageData>, Error> {
    sprites
        .into_iter()
        .map(|path| {
            let (name, _) = path
                .file_name()
                .and_then(OsStr::to_str)
                .and_then(|filename| filename.rsplit_once('.'))
                .unwrap_or_default();

            Ok(ImageData {
                name: name.to_owned().into_boxed_str(),
                data: read_data(Some(path))?,
            })
        })
        .collect()
}

fn make_outdir(outdir: Option<PathBuf>) -> Result<PathBuf, Error> {
    let outdir = outdir
        .or_else(|| env::current_dir().ok())
        .ok_or(Error::OutDir)?;

    if !outdir.exists() {
        fs::create_dir_all(&outdir).map_err(|_| Error::OutDir)?;
    }

    Ok(outdir)
}

fn serialize_elements(elements: &[Element], outdir: &Path) -> Result<(), Error> {
    for Element { name, val } in elements {
        let mut path = outdir.join(name);
        path.set_extension("json");
        println!("write element to file {path:?}");
        let file = {
            let file = File::create(&path).map_err(|_| Error::CreateFile(path))?;
            BufWriter::new(file)
        };

        match val {
            Value::Mesh(mesh) => serde_json::to_writer(file, &mesh),
            Value::Skeleton(sk) => serde_json::to_writer(file, sk.bones()),
            Value::Action(act) => serde_json::to_writer(file, act.keyframes()),
        }
        .expect("serialize element");
    }

    Ok(())
}

fn serialize_colors(colors: &[Color], name: &str, outdir: &Path) -> Result<(), Error> {
    let mut path = outdir.join(name);
    path.set_extension("json");
    println!("write colors ({}) to file {path:?}", colors.len());
    let file = {
        let file = File::create(&path).map_err(|_| Error::CreateFile(path))?;
        BufWriter::new(file)
    };

    serde_json::to_writer(file, colors).expect("serialize colors");
    Ok(())
}

fn serialize_map(map: &Map, name: &str, outdir: &Path) -> Result<(), Error> {
    let mut path = outdir.join(name);
    path.set_extension("json");
    println!("write atlas map to file {path:?}");
    let file = {
        let file = File::create(&path).map_err(|_| Error::CreateFile(path))?;
        BufWriter::new(file)
    };

    serde_json::to_writer(file, map).expect("serialize colors");
    Ok(())
}

fn write_png(data: &[u8], name: &str, outdir: &Path) -> Result<(), Error> {
    let mut path = outdir.join(name);
    path.set_extension("png");
    println!("write image to file {path:?}");
    let mut file = {
        let file = File::create(&path).map_err(|_| Error::CreateFile(path.clone()))?;
        BufWriter::new(file)
    };

    file.write_all(data).map_err(|_| Error::WriteToFile(path))
}

enum Error {
    ReadFile(PathBuf),
    ReadStdin,
    OutDir,
    CreateFile(PathBuf),
    WriteToFile(PathBuf),
    PalettePathNotSet,
    Atlas(AtlasError),
    Parse(ParseError),
    Color(ColorError),
    Json(JsonError),
}

impl From<AtlasError> for Error {
    fn from(v: AtlasError) -> Self {
        Self::Atlas(v)
    }
}

impl From<ParseError> for Error {
    fn from(v: ParseError) -> Self {
        Self::Parse(v)
    }
}

impl From<ColorError> for Error {
    fn from(v: ColorError) -> Self {
        Self::Color(v)
    }
}

impl From<JsonError> for Error {
    fn from(v: JsonError) -> Self {
        Self::Json(v)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ReadFile(path) => write!(f, "failed to read file {path:?}"),
            Self::ReadStdin => write!(f, "failed to read stdin"),
            Self::OutDir => write!(f, "failed to get output directory"),
            Self::CreateFile(path) => write!(f, "failed to create the file {path:?}"),
            Self::WriteToFile(path) => write!(f, "failed to write file {path:?}"),
            Self::PalettePathNotSet => write!(f, "the palette path is not set"),
            Self::Atlas(err) => write!(f, "{err}"),
            Self::Parse(err) => write!(f, "{err}"),
            Self::Color(err) => write!(f, "{err}"),
            Self::Json(err) => write!(f, "{err}"),
        }
    }
}

fn update<const D: u32>(v: f32) -> f32 {
    let a = u32::pow(10, D) as f32;
    let mut v = (v * a).round() / a;
    if v == -0. {
        v = 0.;
    }

    v
}
