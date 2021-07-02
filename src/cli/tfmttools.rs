use crate::cli::argparse;
use anyhow::Result;

pub fn main() -> Result<()> {
    argparse::parse_args()
}
