//! Small CPAL compatibility helpers shared by native adapters and examples.

use cpal::traits::DeviceTrait;
use cpal::{
    Device, Error, ErrorKind, SampleFormat, SupportedStreamConfig, SupportedStreamConfigRange,
};

/// Return the user-facing device name without panicking if the device disappears.
pub fn device_name(device: &Device) -> String {
    device
        .description()
        .map(|description| description.name().to_owned())
        .unwrap_or_default()
}

/// Match current display names plus stable CPAL IDs and pre-0.18 backend IDs.
pub fn device_matches(device: &Device, selection: &str) -> bool {
    device_name(device) == selection
        || device
            .id()
            .is_ok_and(|id| id.id() == selection || id.to_string() == selection)
}

/// Keep input selection within the sample formats the caller actually handles.
pub fn preferred_input_config(
    device: &Device,
    formats: &[SampleFormat],
) -> Result<SupportedStreamConfig, Error> {
    let default = device.default_input_config()?;
    if formats.contains(&default.sample_format()) {
        return Ok(default);
    }
    select_config(default, device.supported_input_configs()?, formats)
}

/// Keep output selection within the sample formats the caller actually handles.
pub fn preferred_output_config(
    device: &Device,
    formats: &[SampleFormat],
) -> Result<SupportedStreamConfig, Error> {
    let default = device.default_output_config()?;
    if formats.contains(&default.sample_format()) {
        return Ok(default);
    }
    select_config(default, device.supported_output_configs()?, formats)
}

fn select_config(
    default: SupportedStreamConfig,
    supported: impl Iterator<Item = SupportedStreamConfigRange>,
    formats: &[SampleFormat],
) -> Result<SupportedStreamConfig, Error> {
    let range = supported
        .filter(|range| formats.contains(&range.sample_format()))
        .max_by(|a, b| {
            let a_rank = formats
                .iter()
                .position(|format| *format == a.sample_format())
                .unwrap_or(formats.len());
            let b_rank = formats
                .iter()
                .position(|format| *format == b.sample_format())
                .unwrap_or(formats.len());
            b_rank
                .cmp(&a_rank)
                .then_with(|| a.cmp_default_heuristics(b))
        })
        .ok_or_else(|| {
            Error::with_message(
                ErrorKind::UnsupportedConfig,
                format!("device does not support any handled sample format: {formats:?}"),
            )
        })?;

    Ok(range
        .try_with_sample_rate(default.sample_rate())
        .or_else(|| range.try_with_standard_sample_rate())
        .unwrap_or_else(|| range.with_max_sample_rate()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cpal::SupportedBufferSize;

    fn range(format: SampleFormat) -> SupportedStreamConfigRange {
        SupportedStreamConfigRange::new(2, 44_100, 48_000, SupportedBufferSize::Unknown, format)
    }

    #[test]
    fn replaces_a_new_integer_default_with_the_callers_preferred_format() {
        let default = range(SampleFormat::I32)
            .try_with_sample_rate(48_000)
            .unwrap();
        let selected = select_config(
            default,
            [range(SampleFormat::I16), range(SampleFormat::F32)].into_iter(),
            &[SampleFormat::F32, SampleFormat::I16],
        )
        .unwrap();

        assert_eq!(selected.sample_format(), SampleFormat::F32);
        assert_eq!(selected.sample_rate(), 48_000);
        assert_eq!(selected.buffer_size(), &SupportedBufferSize::Unknown);
    }

    #[test]
    fn rejects_devices_without_a_handled_sample_format() {
        let default = range(SampleFormat::I32)
            .try_with_sample_rate(48_000)
            .unwrap();
        let error = select_config(
            default,
            [range(SampleFormat::F64)].into_iter(),
            &[SampleFormat::F32],
        )
        .unwrap_err();

        assert_eq!(error.kind(), ErrorKind::UnsupportedConfig);
    }
}
