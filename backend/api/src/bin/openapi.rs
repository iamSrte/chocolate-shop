//! Prints the OpenAPI spec as JSON. Used by `just openapi` to regenerate `frontend/api-client`.

fn main() -> anyhow::Result<()> {
    println!("{}", api::openapi().to_pretty_json()?);
    Ok(())
}
