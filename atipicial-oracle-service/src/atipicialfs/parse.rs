use super::{AtipicialFsCommand, AtipicialFsRange, AtipicialFsRequest, decode_raw_base58};
use atipicial_error::{CoreError, CoreResult};

impl AtipicialFsRequest {
    pub(super) fn parse_atipicialfs_request(url: &str) -> CoreResult<AtipicialFsRequest> {
        let (_, suffix) = url
            .split_once(':')
            .ok_or_else(|| CoreError::other("Invalid atipicialfs url"))?;
        let mut path = suffix;
        if let Some((before, _)) = path.split_once('?') {
            path = before;
        }
        if let Some((before, _)) = path.split_once('#') {
            path = before;
        }
        let segments: Vec<&str> = path.split('/').collect();
        if segments.len() < 2 {
            return Err(CoreError::other("Invalid atipicialfs url"));
        }

        let container = segments[0].to_string();
        let object = segments[1].to_string();
        if container.is_empty() || object.is_empty() {
            return Err(CoreError::other("Invalid atipicialfs url"));
        }
        validate_atipicialfs_id(&container, "container")?;
        validate_atipicialfs_id(&object, "object")?;

        if segments.len() == 2 {
            return Ok(AtipicialFsRequest {
                container,
                object,
                command: AtipicialFsCommand::Payload,
            });
        }

        let command = segments[2];
        let command = match command {
            "range" => {
                let range_raw = segments.get(3).ok_or_else(|| {
                    CoreError::other("missing object range (expected 'Offset|Length')")
                })?;
                AtipicialFsCommand::Range(AtipicialFsRange::parse_atipicialfs_range(range_raw)?)
            }
            "header" => AtipicialFsCommand::Header,
            "hash" => {
                let range = match segments.get(3) {
                    Some(raw) => Some(AtipicialFsRange::parse_atipicialfs_range(raw)?),
                    None => None,
                };
                AtipicialFsCommand::Hash(range)
            }
            _ => return Err(CoreError::other("invalid command")),
        };

        Ok(AtipicialFsRequest {
            container,
            object,
            command,
        })
    }
}

impl AtipicialFsRange {
    pub(super) fn parse_atipicialfs_range(raw: &str) -> CoreResult<AtipicialFsRange> {
        let decoded = percent_encoding::percent_decode_str(raw)
            .decode_utf8()
            .map_err(|_| CoreError::other("object range is invalid (expected 'Offset|Length')"))?;
        let (offset_str, length_str) = decoded.split_once('|').ok_or_else(|| {
            CoreError::other("object range is invalid (expected 'Offset|Length')")
        })?;
        let offset = offset_str
            .parse::<u64>()
            .map_err(|_| CoreError::other("object range is invalid (expected 'Offset|Length')"))?;
        let length = length_str
            .parse::<u64>()
            .map_err(|_| CoreError::other("object range is invalid (expected 'Offset|Length')"))?;
        Ok(AtipicialFsRange { offset, length })
    }
}

fn validate_atipicialfs_id(value: &str, kind: &str) -> CoreResult<()> {
    if decode_raw_base58(value, Some(32)).is_none() {
        return Err(CoreError::other(format!("invalid atipicialfs {} id", kind)));
    }
    Ok(())
}
