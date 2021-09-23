use clap::Clap;

#[derive(Clap)]
pub struct Format {
    filename: String,
}

impl Format {
    #[inline]
    pub fn perform(self) {}
}
