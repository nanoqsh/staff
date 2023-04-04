mod format;
mod mesh;
mod params;
mod parser;
mod skeleton;

use {
    crate::{
        params::Parameters,
        parser::{Error as ParseError, Value},
    },
    clap::Parser,
    std::{
        env, fmt,
        fs::{self, File},
        io,
        path::PathBuf,
        process::ExitCode,
    },
};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// File to parse (stdin by default)
    filepath: Option<PathBuf>,

    /// Enable verbore output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> ExitCode {
    if let Err(err) = run(Cli::parse()) {
        eprintln!("error: {err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn run(Cli { filepath, verbose }: Cli) -> Result<(), Error> {
    Parameters::init(Parameters {
        verbose,
        pos_fn: pos,
        map_fn: map,
        rot_fn: rot,
    });

    let src = match filepath {
        Some(path) => fs::read_to_string(&path).map_err(|_| Error::ReadFile(path))?,
        None => io::read_to_string(io::stdin()).map_err(|_| Error::ReadStdin)?,
    };

    let elements = parser::parse(&src).map_err(Error::Parse)?;
    let curr = env::current_dir().map_err(|_| Error::CurrentDir)?;
    for element in elements {
        let mut path = curr.join(element.name);
        path.set_extension("json");
        println!("write element to file {path:?}");
        let file = File::create(&path).map_err(|_| Error::CreateFile(path))?;
        match element.val {
            Value::Mesh(mesh) => serde_json::to_writer(file, &mesh).expect("serialize element"),
            Value::Skeleton(sk) => serde_json::to_writer(file, &sk).expect("serialize element"),
        }
    }

    Ok(())
}

enum Error {
    ReadFile(PathBuf),
    ReadStdin,
    CurrentDir,
    CreateFile(PathBuf),
    Parse(ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ReadFile(path) => write!(f, "failed to read file {path:?}"),
            Self::ReadStdin => write!(f, "failed to read stdin"),
            Self::CurrentDir => write!(f, "failed to get current directory"),
            Self::CreateFile(path) => write!(f, "failed to create the file {path:?}"),
            Self::Parse(err) => write!(f, "{err}"),
        }
    }
}

fn pos(points: [f32; 3]) -> [f32; 3] {
    points.map(update::<4>)
}

fn map([u, v]: [f32; 2]) -> [f32; 2] {
    [u, 1. - v].map(update::<8>)
}

fn rot(points: [f32; 4]) -> [f32; 4] {
    points.map(update::<6>)
}

fn update<const D: u32>(v: f32) -> f32 {
    let a = u32::pow(10, D) as f32;
    let mut v = (v * a).round() / a;
    if v == -0. {
        v = 0.;
    }

    v
}
