use std::cell::OnceCell;

thread_local! {
    static PARAMS: OnceCell<&'static Parameters> = OnceCell::new();
}

pub struct Parameters {
    pub verbose: bool,
    pub pos_fn: fn([f32; 3]) -> [f32; 3],
    pub map_fn: fn([f32; 2]) -> [f32; 2],
    pub rot_fn: fn([f32; 4]) -> [f32; 4],
    pub act_fn: fn([f32; 2]) -> [f32; 2],
    pub bez_fn: fn([f32; 4]) -> [f32; 4],
}

impl Parameters {
    /// Initialize global parameters.
    ///
    /// # Panics
    /// Panics if global parameters is already set.
    pub fn init(self) {
        let val = Box::leak(self.into());
        let set = PARAMS.with(|params| params.set(val));
        assert!(set.is_ok(), "parameters is already set");
    }

    /// Get global parameters.
    ///
    /// # Panics
    /// Panics if global parameters isn't set.
    #[must_use]
    pub fn get() -> &'static Self {
        match PARAMS.with(|params| params.get().copied()) {
            Some(val) => val,
            None => panic!("parameters isn't set"),
        }
    }
}

macro_rules! verbose {
    ($e:literal) => { verbose!($e,) };
    ($e:literal, $( $t:expr ),* $( , )?) => {
        if Parameters::get().verbose {
            println!($e, $( $t ),*);
        }
    };
}

pub(crate) use verbose;
