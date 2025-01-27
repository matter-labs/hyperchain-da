use anyhow::Context;
use zksync_protobuf::repr::ProtoRepr;

pub fn try_parse_proto_config<T: ProtoRepr>() -> Result<Option<T::Type>, anyhow::Error> {
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg.eq_ignore_ascii_case("--config-path") {
            let path = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("No value for --config-path"))?;

            let yaml = std::fs::read_to_string(path.clone()).with_context(|| path)?;

            let d = serde_yaml::Deserializer::from_str(yaml.as_str());
            let this: T = zksync_protobuf::serde::deserialize_proto_with_options(d, false)?;

            return Ok(Some(this.read()?));
        }
    }

    Ok(None)
}
