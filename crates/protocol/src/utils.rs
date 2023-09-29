use std::str::FromStr;

pub fn convert_timestamp_to_datetime(
  timestamp: &Option<prost_types::Timestamp>,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, astro_run::Error> {
  let res = match timestamp {
    Some(t) => Some(
      chrono::DateTime::from_str(&t.to_string())
        .map_err(|_| astro_run::Error::internal_runtime_error("Invalid timestamp"))?,
    ),
    None => None,
  };

  Ok(res)
}

pub fn convert_datetime_to_timestamp(
  datetime: &Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Option<prost_types::Timestamp>, astro_run::Error> {
  let res = match datetime {
    Some(t) => Some(
      prost_types::Timestamp::from_str(&t.to_rfc3339())
        .map_err(|_| astro_run::Error::internal_runtime_error("Invalid timestamp"))?,
    ),
    None => None,
  };

  Ok(res)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_convert_datatime_to_timestamp() {
    let datetime = chrono::DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z").unwrap();
    let timestamp = convert_datetime_to_timestamp(&Some(datetime.into())).unwrap();
    assert_eq!(timestamp.unwrap().seconds, 1609459200);
  }

  #[test]
  fn test_convert_timestamp_to_datetime() {
    let timestamp = prost_types::Timestamp {
      seconds: 1609459200,
      nanos: 0,
    };
    let datetime = convert_timestamp_to_datetime(&Some(timestamp)).unwrap();
    assert_eq!(datetime.unwrap().to_rfc3339(), "2021-01-01T00:00:00+00:00");
  }

  #[test]
  fn test_convert_datatime_to_timestamp_none() {
    let timestamp = convert_datetime_to_timestamp(&None).unwrap();
    assert_eq!(timestamp, None);
  }

  #[test]
  fn test_convert_timestamp_to_datetime_none() {
    let datetime = convert_timestamp_to_datetime(&None).unwrap();
    assert_eq!(datetime, None);
  }
}
