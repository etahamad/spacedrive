use std::fmt::Display;

use chrono::{DateTime, FixedOffset, NaiveDateTime};

use crate::{
	consts::{OFFSET_TAGS, TIME_TAGS},
	Error, ExifReader,
};

pub const NAIVE_FORMAT_STR: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
/// This can be either naive with no TZ (e.g. `YYYY-MM-DDTHH-MM-SS`) or UTC with a fixed offset (`rfc3339`).
///
/// This may also be `undefined`.
pub enum MediaTime {
	Naive(NaiveDateTime),
	Utc(DateTime<FixedOffset>),
	#[default]
	Undefined,
}

impl MediaTime {
	/// This iterates over all 3 pairs of time/offset tags in an attempt to create a UTC time.
	///
	/// If the above fails, we fall back to Naive time - if that's not present this is `Undefined`.
	pub fn from_reader(reader: &ExifReader) -> Self {
		let z = TIME_TAGS
			.into_iter()
			.zip(OFFSET_TAGS.into_iter())
			.filter_map(|(time_tag, offset_tag)| {
				let time = reader.get_tag::<String>(time_tag);
				let offset = reader.get_tag::<String>(offset_tag);

				if let (Some(t), Some(o)) = (time.clone(), offset) {
					DateTime::parse_and_remainder(&format!("{t} {o}"), "%F %X %#z")
						.ok()
						.map(|x| Self::Utc(x.0))
				} else if let Some(t) = time {
					Some(
						NaiveDateTime::parse_from_str(&t, NAIVE_FORMAT_STR)
							.map_or(Self::Undefined, Self::Naive),
					)
				} else {
					Some(Self::Undefined)
				}
			})
			.collect::<Vec<_>>();

		z.iter()
			.find(|x| match x {
				Self::Utc(_) | Self::Naive(_) => true,
				Self::Undefined => false,
			})
			.map_or(Self::Undefined, Clone::clone)
	}
}

impl TryFrom<String> for MediaTime {
	type Error = Error;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		if &value == "Undefined" {
			return Ok(Self::Undefined);
		}

		if let Ok(time) = DateTime::parse_from_rfc3339(&value) {
			return Ok(Self::Utc(time));
		}

		Ok(NaiveDateTime::parse_from_str(&value, NAIVE_FORMAT_STR)
			.map_or(Self::Undefined, Self::Naive))
	}
}

impl Display for MediaTime {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Undefined => f.write_str("Undefined"),
			Self::Naive(l) => f.write_str(&l.to_string()),
			Self::Utc(u) => f.write_str(&u.to_rfc3339()),
		}
	}
}
