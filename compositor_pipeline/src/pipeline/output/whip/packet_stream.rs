use std::collections::VecDeque;

use crossbeam_channel::Receiver;

use crate::pipeline::types::EncoderOutputEvent;

use super::payloader::{Payload, Payloader, PayloadingError};

pub(super) struct PacketStream {
    packets_receiver: Receiver<EncoderOutputEvent>,
    state: VecDeque<Payload>,
    payloader: Payloader,
    mtu: usize,
}

impl PacketStream {
    pub(super) fn new(
        packets_receiver: Receiver<EncoderOutputEvent>,
        payloader: Payloader,
        mtu: usize,
    ) -> Self {
        Self {
            packets_receiver,
            payloader,
            mtu,
            state: VecDeque::new(),
        }
    }

    fn next_new_packet(&mut self) -> Option<Result<Payload, PayloadingError>> {
        let Ok(packet) = self.packets_receiver.recv() else {
            // Send audio and video EOS if payloaders are supported and EOS was not sent before.
            match self.payloader.audio_eos() {
                Err(PayloadingError::NoAudioPayloader) => (),
                Err(PayloadingError::AudioEOSAlreadySent) => (),
                packet => return Some(Ok(Payload::Audio(packet))),
            }
            match self.payloader.video_eos() {
                Err(PayloadingError::NoVideoPayloader) => (),
                Err(PayloadingError::VideoEOSAlreadySent) => (),
                packet => return Some(Ok(Payload::Video(packet))),
            }
            return None;
        };

        let encoded_chunk = match packet {
            EncoderOutputEvent::Data(packet) => packet,
            EncoderOutputEvent::AudioEOS => {
                return Some(Ok(Payload::Audio(self.payloader.audio_eos())));
            }
            EncoderOutputEvent::VideoEOS => {
                return Some(Ok(Payload::Video(self.payloader.video_eos())));
            }
        };

        let rtp_packets = match self.payloader.payload(self.mtu, encoded_chunk) {
            Ok(packets) => packets,
            Err(err) => return Some(Err(err)),
        };

        // I'm assuming here that payload will never return empty list
        self.state = rtp_packets;
        self.state.pop_front().map(Ok)
    }
}

impl Iterator for PacketStream {
    type Item = Result<Payload, PayloadingError>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            chunks if chunks.is_empty() => self.next_new_packet(),
            chunks => chunks.pop_front().map(Ok),
        }
    }
}
