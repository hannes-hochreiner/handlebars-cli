use std::env;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;

use handlebars::Handlebars;
use serde_json::value::Value as Json;
use snafu::{ResultExt, Snafu};

/// An enumeration of the possible errors this program may encounter.
#[derive(Debug, Snafu)]
enum Error {
    /// This error represents the properties being parsed as invalid JSON.
    #[snafu(display("Unable to parse properties JSON: {}", source))]
    PropsInvalidJson { source: serde_json::Error },

    /// This error represents the Handlebars template not being found at the provided path.
    #[snafu(display("Unable to read template from '{}'.", path))]
    TemplateNotFound { path: String },

    /// This error represents the Handlebars template not being valid Handlebars syntax.
    #[snafu(display("File at '{}' was not a valid handlebars template: {}", path.display(), source))]
    TemplateInvalid {
        source: handlebars::TemplateError,
        path: PathBuf,
    },

    /// This error represents the Handlebars template attempting to use properties not provided,
    /// and so rendering failed.
    #[snafu(display("Template at '{}' failed to render: {}", path.display(), source))]
    TemplateRenderFailed {
        source: handlebars::RenderError,
        path: PathBuf,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

static USAGE: &str =
    "handlebars-cli — Template JSON properties into Handlebars templates from the CLI.

USAGE:
    handlebars-cli <JSON> <TEMPLATE>
    handlebars-cli --help

PARAMETERS:
    JSON: A set of valid JSON to use as properties to interpolate into the provided template file.
    TEMPLATE: A path to a valid Handlebars template.

FLAGS:
    --help: Prints this usage text.";

fn main() -> () {
    let mut args = env::args();
    args.next(); // skip own filename

    let (raw_props, raw_filename) = match (args.next(), args.next()) {
        (Some(raw_props), Some(raw_filename)) => (raw_props, raw_filename),
        _ => {
            eprintln!("{}", USAGE);
            process::exit(1);
        }
    };

    match execute_handlebars_templating(raw_props, raw_filename) {
        Ok(data) => {
            println!("{}", data)
        }
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1)
        }
    }
}

/// Given a string which should contain valid JSON representing a set of properties, take those
/// properties and interpolate them into a handlebars template at the given path.
///
/// If everything succeeds, this will return the templated result.
///
/// It fails if the properties are not valid JSON.
/// It fails if the template file could not be found.
/// It fails if the template file is not a valid Handlebars template.
/// It fails if the template file used properties that were not available.
fn execute_handlebars_templating(raw_props: String, raw_filename: String) -> Result<String, Error> {
    let props = Json::from_str(&raw_props).context(PropsInvalidJsonSnafu {})?;

    let filename = PathBuf::from(&raw_filename);
    if !filename.exists() {
        return TemplateNotFoundSnafu { path: raw_filename }.fail();
    }

    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);

    handlebars
        .register_template_file(&raw_filename, &filename)
        .context(TemplateInvalidSnafu { path: &filename })?;

    handlebars
        .render(&raw_filename, &props)
        .context(TemplateRenderFailedSnafu { path: &filename })
}
