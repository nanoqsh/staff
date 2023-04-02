mod format;
mod mesh;
#[allow(dead_code)]
mod parser;
mod skeleton;

use {
    crate::parser::{Error as ParseError, Value},
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
    use crate::parser::Parameters;

    let src = match filepath {
        Some(path) => fs::read_to_string(&path).map_err(|_| Error::ReadFile(path))?,
        None => io::read_to_string(io::stdin()).map_err(|_| Error::ReadStdin)?,
    };

    let params = Parameters {
        verbose,
        pos_fn: &pos,
        map_fn: &map,
    };

    let elements = parser::parse(params, &src).map_err(Error::Parse)?;
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
    const DECIMALS: u32 = 4;
    const ACCURACY: u32 = u32::pow(10, DECIMALS);

    let a = ACCURACY as f32;
    let update = |mut v: f32| {
        if v == -0. {
            v = 0.;
        }

        (v * a).round() / a
    };

    points.map(update)
}

fn map([u, v]: [f32; 2]) -> [f32; 2] {
    [u, 1. - v]
}
