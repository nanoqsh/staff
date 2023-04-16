use {
    clap::{Parser, Subcommand},
    convert::{Element, Error as ParseError, Parameters, Target, Value},
    std::{
        env, fmt,
        fs::{self, File},
        io::{self, BufWriter},
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
}

fn main() -> ExitCode {
    if let Err(err) = run(Cli::parse()) {
        eprintln!("error: {err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn run(cli: Cli) -> Result<(), Error> {
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
            let src = match filepath {
                Some(path) => fs::read_to_string(&path).map_err(|_| Error::ReadFile(path))?,
                None => io::read_to_string(io::stdin()).map_err(|_| Error::ReadStdin)?,
            };

            let elements = convert::parse(&src, target).map_err(Error::Parse)?;
            if elements.is_empty() {
                println!("no elements found");
                return Ok(());
            }

            let outdir = outdir
                .or_else(|| env::current_dir().ok())
                .ok_or(Error::OutDir)?;

            if !outdir.exists() {
                fs::create_dir_all(&outdir).map_err(|_| Error::OutDir)?;
            }

            serialize(&elements, &outdir)
        }
    }
}

fn serialize(elements: &[Element], outdir: &Path) -> Result<(), Error> {
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
            Value::Action(act) => serde_json::to_writer(file, act.animations()),
        }
        .expect("serialize element");
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
