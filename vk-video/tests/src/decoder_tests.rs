// TODO: tests
// - changing resolution low -> high -> low
// - b frames
// - missed frame
// - what about h264 which crashes the machine?
// - separate tests for bytes and textures decoder

use std::{fs, path::PathBuf, str::FromStr, sync::Arc};

use vk_video::{
    DecoderError, EncodedInputChunk, OutputFrame, RawFrameData, VulkanDevice, VulkanInstance,
    parameters::{DecoderParameters, VulkanAdapterDescriptor, VulkanDeviceDescriptor},
};

#[test]
fn h264_dynamic_resolution() {
    run_decoder_test(
        "decoder/h264/dynamic_resolution",
        InputByteStream::H264(include_bytes!(
            "../fixtures/dumps/h264/dynamic_resolution.h264"
        )),
        &[],
        0.0,
    );
}

fn run_decoder_test(
    name: &str,
    input_bytestream: InputByteStream<'_>,
    frames_to_verify: &[usize],
    allowed_error: f32,
) {
    // TODO: verify name uniquness? (maybe fetch the name from the test function name?? (if possible))
    // TODO: maybe share device between tests?
    let device = create_device(VulkanAdapterDescriptor {
        supports_decoding: true,
        supports_encoding: false,
        compatible_surface: None,
    });

    let frames = match input_bytestream {
        InputByteStream::H264(data) => {
            // TODO: test texture decoder?
            device
                .create_bytes_decoder(DecoderParameters::default())
                .unwrap()
                .decode(EncodedInputChunk { data, pts: None })
                .unwrap()
        }
    };

    for frame in frames {
        let diff = calculate_snapshot_diff(name, frame, 0);
        if diff > allowed_error {
            // TODO: update or fail
        }
    }
}

fn calculate_snapshot_diff(name: &str, frame: OutputFrame<RawFrameData>, frame_id: usize) -> f32 {
    let snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("snapshots")
        .join(format!("{name}_{frame_id}.nv12"));
    let Ok(snapshot) = fs::read(snapshot_path) else {
        println!(
            "snapshot not found, generate it with --feature update_snapshots: {name}_{frame_id}.nv12"
        );
        return f32::MAX;
    };

    if frame.data.frame.len() != snapshot.len() {
        return f32::MAX;
    }

    let diff = frame
        .data
        .frame
        .iter()
        .zip(&snapshot)
        .fold(0.0, |total_err, (a, b)| {
            total_err + (*a as f32 - *b as f32).powf(2.0)
        });

    diff / snapshot.len() as f32
}

fn create_device(descriptor: VulkanAdapterDescriptor) -> Arc<VulkanDevice> {
    let instance = VulkanInstance::new().unwrap();
    let adapter = instance.create_adapter(&descriptor).unwrap();
    adapter
        .create_device(&VulkanDeviceDescriptor::default())
        .unwrap()
}

// TODO: maybe change it once we have more codecs support added
enum InputByteStream<'a> {
    H264(&'a [u8]),
}
