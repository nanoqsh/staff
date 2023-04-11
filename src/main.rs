use {
    clap::Parser,
    convert::{Error as ParseError, Parameters, Value},
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

    /// Specify output directory (current by default)
    #[arg(short, long)]
    outdir: Option<PathBuf>,

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

fn run(cli: Cli) -> Result<(), Error> {
    Parameters::init(Parameters {
        verbose: cli.verbose,
        pos_fn: |vs| vs.map(update::<4>),
        map_fn: |[u, v]| [u, 1. - v].map(update::<8>),
        rot_fn: |vs| vs.map(update::<6>),
    });

    let src = match cli.filepath {
        Some(path) => fs::read_to_string(&path).map_err(|_| Error::ReadFile(path))?,
        None => io::read_to_string(io::stdin()).map_err(|_| Error::ReadStdin)?,
    };

    let path = cli
        .outdir
        .or_else(|| env::current_dir().ok())
        .ok_or(Error::OutDir)?;

    if !path.exists() {
        fs::create_dir_all(&path).map_err(|_| Error::OutDir)?;
    }

    let elements = convert::parse(&src).map_err(Error::Parse)?;
    for element in elements {
        let mut path = path.join(element.name);
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
    OutDir,
    CreateFile(PathBuf),
    Parse(ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ReadFile(path) => write!(f, "failed to read file {path:?}"),
            Self::ReadStdin => write!(f, "failed to read stdin"),
            Self::OutDir => write!(f, "failed to get output directory"),
            Self::CreateFile(path) => write!(f, "failed to create the file {path:?}"),
            Self::Parse(err) => write!(f, "{err}"),
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
