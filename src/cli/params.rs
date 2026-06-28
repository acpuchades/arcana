//! `--param NAME=value` substitution for `strategy.yml`.
//!
//! The strategy spec ([`crate::spec`]) deserializes into strongly-typed serde
//! enums, where a `period` is a `usize`, a `k` is a `Real`, and so on — there is
//! no room to drop a `!param` tag where a number is expected during typed
//! parsing. So substitution happens in a **first pass over the untyped YAML
//! tree**: parse the document into a [`serde_norway::Value`], rewrite every
//! `!param` node into its resolved value here, then `from_value` the result into
//! the typed spec. The injected value is an already-parsed YAML scalar, so the
//! typed parse stays type-correct and every other tag (`!sma`, `!crosses_above`,
//! …) still resolves to its enum variant.
//!
//! A placeholder takes either form:
//!
//! ```yaml
//! period: !param { key: FAST }                # required — must be passed
//! period: !param { key: SLOW, default: 8 }    # optional — falls back to 8
//! symbol: !param SYM                           # bare-string shorthand for { key: SYM }
//! ```

use std::collections::HashMap;

use anyhow::{Context, Result, anyhow, bail};
use serde_norway::Value;
use serde_norway::value::TaggedValue;

/// Parse `--param NAME=value` arguments into a name → value table.
///
/// The value side is parsed as a YAML scalar, so `FAST=3` is an integer,
/// `K=2.0` a float and `SYM=BTC` a string — each lands in the spec with the
/// type its target field expects.
pub fn parse(args: &[String]) -> Result<HashMap<String, Value>> {
    let mut map = HashMap::new();
    for arg in args {
        let (name, raw) = arg
            .split_once('=')
            .ok_or_else(|| anyhow!("invalid --param `{arg}`: expected NAME=value"))?;
        let value: Value = serde_norway::from_str(raw)
            .with_context(|| format!("parsing value of --param `{name}`"))?;
        map.insert(name.to_string(), value);
    }
    Ok(map)
}

/// Rewrite every `!param` node in `value` to its resolved value, recursing
/// through mappings, sequences and other tagged nodes.
pub fn substitute(value: Value, params: &HashMap<String, Value>) -> Result<Value> {
    match value {
        Value::Tagged(tagged) if tagged.tag == "param" => resolve(&tagged.value, params),
        Value::Tagged(tagged) => {
            // A non-`param` tag (e.g. `!sma`): keep the tag, substitute inside it
            // so a `!param` nested in its body is still resolved.
            let value = substitute(tagged.value, params)?;
            Ok(Value::Tagged(Box::new(TaggedValue {
                tag: tagged.tag,
                value,
            })))
        }
        Value::Sequence(seq) => seq
            .into_iter()
            .map(|v| substitute(v, params))
            .collect::<Result<Vec<_>>>()
            .map(Value::Sequence),
        Value::Mapping(map) => {
            let mut out = serde_norway::Mapping::new();
            for (k, v) in map {
                // Keys are not templated, only values.
                out.insert(k, substitute(v, params)?);
            }
            Ok(Value::Mapping(out))
        }
        other => Ok(other),
    }
}

/// Resolve a single `!param` body (its `{ key, default }` mapping or bare key
/// name) against the supplied params.
fn resolve(spec: &Value, params: &HashMap<String, Value>) -> Result<Value> {
    let (key, default) = match spec {
        Value::String(name) => (name.as_str(), None),
        Value::Mapping(_) => {
            let key = spec
                .get("key")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("`!param` mapping needs a string `key`"))?;
            (key, spec.get("default"))
        }
        _ => bail!("`!param` expects a key name or a `{{ key: NAME }}` mapping"),
    };

    if let Some(value) = params.get(key) {
        Ok(value.clone())
    } else if let Some(default) = default {
        Ok(default.clone())
    } else {
        bail!("parameter `{key}` is not set (pass `--param {key}=…` or add a `default`)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::StrategySpec;

    fn params(pairs: &[&str]) -> HashMap<String, Value> {
        parse(&pairs.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap()
    }

    #[test]
    fn parse_types_values_as_yaml_scalars() {
        let map = params(&["FAST=3", "K=2.0", "SYM=BTC"]);
        assert_eq!(map["FAST"], Value::from(3));
        assert_eq!(map["K"], Value::from(2.0));
        assert_eq!(map["SYM"], Value::from("BTC"));
    }

    #[test]
    fn parse_rejects_missing_equals() {
        assert!(parse(&["FAST".to_string()]).is_err());
    }

    fn sub(yaml: &str, pairs: &[&str]) -> Result<Value> {
        let value: Value = serde_norway::from_str(yaml).unwrap();
        substitute(value, &params(pairs))
    }

    #[test]
    fn provided_value_wins_over_default() {
        let out = sub("period: !param { key: FAST, default: 8 }", &["FAST=3"]).unwrap();
        assert_eq!(out.get("period"), Some(&Value::from(3)));
    }

    #[test]
    fn falls_back_to_default_when_unset() {
        let out = sub("period: !param { key: FAST, default: 8 }", &[]).unwrap();
        assert_eq!(out.get("period"), Some(&Value::from(8)));
    }

    #[test]
    fn errors_when_unset_and_no_default() {
        let err = sub("period: !param { key: FAST }", &[]).unwrap_err();
        assert!(err.to_string().contains("FAST"));
    }

    #[test]
    fn bare_string_shorthand() {
        let out = sub("symbol: !param SYM", &["SYM=ETH"]).unwrap();
        assert_eq!(out.get("symbol"), Some(&Value::from("ETH")));
    }

    #[test]
    fn leaves_other_tags_intact_and_round_trips_into_a_strategy() {
        // The key risk: after walking the Value tree, the surviving `!sma` /
        // `!crosses_above` tags must still resolve to their enum variants when we
        // re-`from_value` into the typed spec.
        let yaml = r#"
            symbol: !param { key: SYM, default: BTC }
            long:
              enter: !crosses_above
                lhs: !sma { source: close, period: !param { key: FAST } }
                rhs: !sma { source: close, period: !param { key: SLOW, default: 8 } }
            short:
              enter: !crosses_below
                lhs: !sma { source: close, period: !param { key: FAST } }
                rhs: !sma { source: close, period: !param { key: SLOW, default: 8 } }
        "#;
        let value: Value = serde_norway::from_str(yaml).unwrap();
        let value = substitute(value, &params(&["FAST=3"])).unwrap();
        let spec: StrategySpec = serde_norway::from_value(value).unwrap();
        assert_eq!(spec.symbol, "BTC");
        assert!(spec.long.is_some());
        let _strat = spec.build();
    }
}
