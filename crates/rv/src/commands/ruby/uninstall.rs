use anstream::println;
use camino::Utf8PathBuf;
use owo_colors::OwoColorize;
use rv_ruby::request::RubyRequest;

use crate::config::Config;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error("no matching ruby version found")]
    NoMatchingRuby,
    #[error(transparent)]
    ConfigError(#[from] crate::config::Error),
    #[error("Could not delete dir {dir}: {error}")]
    IoError {
        dir: Utf8PathBuf,
        error: std::io::Error,
    },
}

type Result<T> = miette::Result<T, Error>;

/// Uninstall the given Ruby version.
pub async fn uninstall(config: &Config, request: RubyRequest) -> Result<()> {
    if let Some(ruby) = config.matching_ruby(&request) {
        let ruby_path = ruby.path;
        println!("Deleting {}", ruby_path.cyan());

        // Delete the dir at this Ruby version's path.
        fs_err::remove_dir_all(&ruby_path).map_err(|error| Error::IoError {
            dir: ruby_path,
            error,
        })?;
        Ok(())
    } else {
        Err(Error::NoMatchingRuby)
    }
}
