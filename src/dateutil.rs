use lazy_static::lazy_static;
use serde::Serializer;
use time::format_description::OwnedFormatItem;
use time::{format_description, OffsetDateTime};

lazy_static! {
    static ref FD: OwnedFormatItem =
        format_description::parse_owned::<2>("[year]/[month]/[day] [hour]:[minute]:[second]",)
            .unwrap();
}

pub fn date_as_human_friendly<S: Serializer>(
    date: &OffsetDateTime,
    s: S,
) -> Result<S::Ok, S::Error> {
    let datestr = date
        .format(&FD)
        .map_err(|e| serde::ser::Error::custom(format!("{e:?}")))?;
    s.serialize_str(&datestr)
}

pub fn date_option_as_human_friendly<S: Serializer>(
    date: &Option<OffsetDateTime>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match date {
        Some(date) => date_as_human_friendly(date, s),
        None => s.serialize_str(""),
    }
}
