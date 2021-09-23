use clap::Clap;

#[derive(Clap)]
pub struct Check {
    filename: String,
}

impl Check {
    #[inline]
    pub fn perform(self) {}
}
