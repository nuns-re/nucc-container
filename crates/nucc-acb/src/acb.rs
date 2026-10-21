use std::collections::HashSet;

use nucc_core::{NuccError, Result};

use crate::utf::UtfTable;

#[derive(Debug, Clone)]
pub struct AcbHeader {
    pub name: String,
    pub version_string: String,
    pub version: Option<u32>,
    pub cue_count: usize,
    pub waveform_count: usize,
    pub awb_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcbReference {
    pub kind: u16,
    pub index: u16,
}

#[derive(Debug, Clone)]
pub struct AcbCue {
    pub index: usize,
    pub reference_type: u8,
    pub reference_index: u16,
    pub num_related_waveforms: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct AcbCueName {
    pub index: usize,
    pub cue_index: u16,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct AcbWaveform {
    pub index: usize,
    pub id: u16,
    pub memory_awb_id: Option<u16>,
    pub stream_awb_id: Option<u16>,
    pub port_no: Option<u16>,
    pub streaming: u8,
    pub loop_flag: u8,
    pub encode_type: Option<u8>,
    pub num_channels: Option<u8>,
    pub sampling_rate: Option<u16>,
    pub num_samples: Option<u32>,
    pub extension_data: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct AcbSynth {
    pub index: usize,
    pub kind: u8,
    pub references: Vec<AcbReference>,
}

#[derive(Debug, Clone)]
pub struct AcbSequence {
    pub index: usize,
    pub kind: u8,
    pub track_indices: Vec<i16>,
}

#[derive(Debug, Clone)]
pub struct AcbTrack {
    pub index: usize,
    pub event_index: u16,
}

#[derive(Debug, Clone)]
pub struct AcbTrackEvent {
    pub index: usize,
    pub references: Vec<AcbReference>,
}

#[derive(Debug, Clone)]
pub struct AcbBlock {
    pub index: usize,
    pub track_indices: Vec<i16>,
}

#[derive(Debug, Clone)]
pub struct AcbBlockSequence {
    pub index: usize,
    pub track_indices: Vec<i16>,
    pub block_indices: Vec<i16>,
}

#[derive(Debug, Clone)]
pub struct AcbWaveformExtension {
    pub index: usize,
    pub loop_start: u32,
    pub loop_end: u32,
}

#[derive(Debug, Clone)]
pub struct Acb {
    pub header: AcbHeader,
    pub utf: UtfTable,
    pub cues: Vec<AcbCue>,
    pub cue_names: Vec<AcbCueName>,
    pub waveforms: Vec<AcbWaveform>,
    pub synths: Vec<AcbSynth>,
    pub sequences: Vec<AcbSequence>,
    pub tracks: Vec<AcbTrack>,
    pub track_events: Vec<AcbTrackEvent>,
    pub blocks: Vec<AcbBlock>,
    pub block_sequences: Vec<AcbBlockSequence>,
    pub waveform_extensions: Vec<AcbWaveformExtension>,
    pub cue_waveforms: Vec<Vec<usize>>,
    awb_offset: Option<usize>,
    awb_size: usize,
}

impl Acb {
    pub fn parse(data: &[u8]) -> Result<Self> {
        let utf = UtfTable::parse(data)?;
        if utf.name() != "Header" || utf.row_count() == 0 {
            return Err(NuccError::InvalidHeader);
        }

        let name = utf.get_string(0, "Name").unwrap_or_default().to_owned();
        let version_string = utf
            .get_string(0, "VersionString")
            .unwrap_or_default()
            .to_owned();
        let version = utf.get_u32(0, "Version");
        let awb = utf
            .get(0, "AwbFile")
            .and_then(|value| value.as_data())
            .filter(|(_, size)| *size != 0);
        let awb_offset = awb.map(|(offset, _)| offset);
        let awb_size = awb.map_or(0, |(_, size)| size);

        let cues = read_cues(&utf)?;
        let cue_names = read_cue_names(&utf)?;
        let waveforms = read_waveforms(&utf)?;
        let synths = read_synths(&utf)?;
        let sequences = read_sequences(&utf)?;
        let tracks = read_tracks(&utf)?;
        let track_events = read_track_events(&utf)?;
        let blocks = read_blocks(&utf)?;
        let block_sequences = read_block_sequences(&utf)?;
        let waveform_extensions = read_waveform_extensions(&utf)?;

        let mut acb = Self {
            header: AcbHeader {
                name,
                version_string,
                version,
                cue_count: cues.len(),
                waveform_count: waveforms.len(),
                awb_size,
            },
            utf,
            cues,
            cue_names,
            waveforms,
            synths,
            sequences,
            tracks,
            track_events,
            blocks,
            block_sequences,
            waveform_extensions,
            cue_waveforms: Vec::new(),
            awb_offset,
            awb_size,
        };
        let cue_waveforms = (0..acb.cues.len())
            .map(|index| acb.waveforms_for_cue_index(index))
            .collect();
        acb.cue_waveforms = cue_waveforms;

        Ok(acb)
    }

    pub fn table(&self, name: &str) -> Result<UtfTable> {
        self.utf.open_subtable(0, name)
    }

    pub fn awb_data(&self) -> Option<&[u8]> {
        self.utf
            .get_data(0, "AwbFile")
            .filter(|data| !data.is_empty())
    }

    #[cfg(feature = "awb")]
    pub fn awb(&self) -> Result<Option<nucc_awb::Awb>> {
        self.awb_data().map(nucc_awb::Awb::parse).transpose()
    }

    pub fn awb_offset(&self) -> Option<usize> {
        self.awb_offset
    }

    pub fn awb_size(&self) -> usize {
        self.awb_size
    }

    pub fn stream_awb_toc(&self) -> Option<&[u8]> {
        self.utf
            .get_data(0, "StreamAwbTocWork")
            .or_else(|| self.utf.get_data(0, "StreamAwbTocWork_Old"))
    }

    pub fn stream_awb_afs2_header(&self) -> Option<&[u8]> {
        self.utf.get_data(0, "StreamAwbAfs2Header")
    }

    pub fn waveforms_for_cue_index(&self, index: usize) -> Vec<usize> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        self.resolve_cue_index(index, 0, &mut seen, &mut result);
        result
    }

    pub fn waveforms_for_cue_name(&self, name: &str) -> Vec<usize> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        for cue_name in self
            .cue_names
            .iter()
            .filter(|cue_name| cue_name.name == name)
        {
            if let Some(indices) = self.cue_waveforms.get(cue_name.cue_index as usize) {
                for &index in indices {
                    if seen.insert(index) {
                        result.push(index);
                    }
                }
            }
        }
        result
    }

    fn resolve_cue_index(
        &self,
        index: usize,
        depth: u8,
        seen: &mut HashSet<usize>,
        result: &mut Vec<usize>,
    ) {
        let Some(cue) = self.cues.get(index) else {
            return;
        };
        if depth > 12 {
            return;
        }

        match cue.reference_type {
            1 => self.add_waveform(cue.reference_index as usize, seen, result),
            2 => self.resolve_synth(cue.reference_index as usize, depth + 1, seen, result),
            3 => self.resolve_sequence(cue.reference_index as usize, depth + 1, seen, result),
            8 => self.resolve_block_sequence(cue.reference_index as usize, depth + 1, seen, result),
            _ => {}
        }
    }

    fn resolve_reference(
        &self,
        reference: AcbReference,
        depth: u8,
        seen: &mut HashSet<usize>,
        result: &mut Vec<usize>,
    ) {
        match reference.kind {
            1 => self.add_waveform(reference.index as usize, seen, result),
            2 => self.resolve_synth(reference.index as usize, depth, seen, result),
            3 => self.resolve_sequence(reference.index as usize, depth, seen, result),
            _ => {}
        }
    }

    fn resolve_synth(
        &self,
        index: usize,
        depth: u8,
        seen: &mut HashSet<usize>,
        result: &mut Vec<usize>,
    ) {
        let Some(synth) = self.synths.get(index) else {
            return;
        };
        if depth > 12 {
            return;
        }
        for reference in &synth.references {
            self.resolve_reference(*reference, depth + 1, seen, result);
        }
    }

    fn resolve_sequence(
        &self,
        index: usize,
        depth: u8,
        seen: &mut HashSet<usize>,
        result: &mut Vec<usize>,
    ) {
        let Some(sequence) = self.sequences.get(index) else {
            return;
        };
        if depth > 12 {
            return;
        }
        for &track_index in &sequence.track_indices {
            if track_index >= 0 {
                self.resolve_track(track_index as usize, depth + 1, seen, result);
            }
        }
    }

    fn resolve_track(
        &self,
        index: usize,
        depth: u8,
        seen: &mut HashSet<usize>,
        result: &mut Vec<usize>,
    ) {
        let Some(track) = self.tracks.get(index) else {
            return;
        };
        if track.event_index == u16::MAX {
            return;
        }
        self.resolve_track_event(track.event_index as usize, depth, seen, result);
    }

    fn resolve_track_event(
        &self,
        index: usize,
        depth: u8,
        seen: &mut HashSet<usize>,
        result: &mut Vec<usize>,
    ) {
        let Some(event) = self.track_events.get(index) else {
            return;
        };
        if depth > 12 {
            return;
        }
        for reference in &event.references {
            self.resolve_reference(*reference, depth + 1, seen, result);
        }
    }

    fn resolve_block(
        &self,
        index: usize,
        depth: u8,
        seen: &mut HashSet<usize>,
        result: &mut Vec<usize>,
    ) {
        let Some(block) = self.blocks.get(index) else {
            return;
        };
        for &track_index in &block.track_indices {
            if track_index >= 0 {
                self.resolve_track(track_index as usize, depth + 1, seen, result);
            }
        }
    }

    fn resolve_block_sequence(
        &self,
        index: usize,
        depth: u8,
        seen: &mut HashSet<usize>,
        result: &mut Vec<usize>,
    ) {
        let Some(sequence) = self.block_sequences.get(index) else {
            return;
        };
        if depth > 12 {
            return;
        }
        for &track_index in &sequence.track_indices {
            if track_index >= 0 {
                self.resolve_track(track_index as usize, depth + 1, seen, result);
            }
        }
        for &block_index in &sequence.block_indices {
            if block_index >= 0 {
                self.resolve_block(block_index as usize, depth + 1, seen, result);
            }
        }
    }

    fn add_waveform(&self, index: usize, seen: &mut HashSet<usize>, result: &mut Vec<usize>) {
        if index < self.waveforms.len() && seen.insert(index) {
            result.push(index);
        }
    }
}

fn read_cues(utf: &UtfTable) -> Result<Vec<AcbCue>> {
    let Some(table) = open_optional_table(utf, "CueTable")? else {
        return Ok(Vec::new());
    };

    let mut cues = Vec::with_capacity(table.row_count());
    for index in 0..table.row_count() {
        cues.push(AcbCue {
            index,
            reference_type: table.get_u8(index, "ReferenceType").unwrap_or(0),
            reference_index: table.get_u16(index, "ReferenceIndex").unwrap_or(0),
            num_related_waveforms: table.get_u16(index, "NumRelatedWaveforms"),
        });
    }
    Ok(cues)
}

fn read_cue_names(utf: &UtfTable) -> Result<Vec<AcbCueName>> {
    let Some(table) = open_optional_table(utf, "CueNameTable")? else {
        return Ok(Vec::new());
    };

    let mut names = Vec::with_capacity(table.row_count());
    for index in 0..table.row_count() {
        names.push(AcbCueName {
            index,
            cue_index: table.get_u16(index, "CueIndex").unwrap_or(0),
            name: table
                .get_string(index, "CueName")
                .unwrap_or_default()
                .to_owned(),
        });
    }
    Ok(names)
}

fn read_waveforms(utf: &UtfTable) -> Result<Vec<AcbWaveform>> {
    let Some(table) = open_optional_table(utf, "WaveformTable")? else {
        return Ok(Vec::new());
    };

    let mut waveforms = Vec::with_capacity(table.row_count());
    for index in 0..table.row_count() {
        let memory_awb_id = table
            .get_u16(index, "MemoryAwbId")
            .filter(|value| *value != u16::MAX);
        let stream_awb_id = table
            .get_u16(index, "StreamAwbId")
            .filter(|value| *value != u16::MAX);
        let id = table
            .get_u16(index, "Id")
            .or(stream_awb_id)
            .or(memory_awb_id)
            .unwrap_or(0);
        let port_no = table
            .get_u16(index, "StreamAwbPortNo")
            .filter(|value| *value != u16::MAX);
        let extension_data = table
            .get_u16(index, "ExtensionData")
            .filter(|value| *value != u16::MAX);
        waveforms.push(AcbWaveform {
            index,
            id,
            memory_awb_id,
            stream_awb_id,
            port_no,
            streaming: table.get_u8(index, "Streaming").unwrap_or(0),
            loop_flag: table.get_u8(index, "LoopFlag").unwrap_or(0),
            encode_type: table.get_u8(index, "EncodeType"),
            num_channels: table.get_u8(index, "NumChannels"),
            sampling_rate: table.get_u16(index, "SamplingRate"),
            num_samples: table.get_u32(index, "NumSamples"),
            extension_data,
        });
    }
    Ok(waveforms)
}

fn read_synths(utf: &UtfTable) -> Result<Vec<AcbSynth>> {
    let Some(table) = open_optional_table(utf, "SynthTable")? else {
        return Ok(Vec::new());
    };

    let mut synths = Vec::with_capacity(table.row_count());
    for index in 0..table.row_count() {
        synths.push(AcbSynth {
            index,
            kind: table.get_u8(index, "Type").unwrap_or(0),
            references: table
                .get_data(index, "ReferenceItems")
                .map(read_references)
                .unwrap_or_default(),
        });
    }
    Ok(synths)
}

fn read_sequences(utf: &UtfTable) -> Result<Vec<AcbSequence>> {
    let Some(table) = open_optional_table(utf, "SequenceTable")? else {
        return Ok(Vec::new());
    };

    let mut sequences = Vec::with_capacity(table.row_count());
    for index in 0..table.row_count() {
        let num_tracks = table.get_u16(index, "NumTracks").unwrap_or(0) as usize;
        let track_indices = table
            .get_data(index, "TrackIndex")
            .map(|data| read_i16_list(data, num_tracks))
            .unwrap_or_default();
        sequences.push(AcbSequence {
            index,
            kind: table.get_u8(index, "Type").unwrap_or(0),
            track_indices,
        });
    }
    Ok(sequences)
}

fn read_tracks(utf: &UtfTable) -> Result<Vec<AcbTrack>> {
    let Some(table) = open_optional_table(utf, "TrackTable")? else {
        return Ok(Vec::new());
    };

    let mut tracks = Vec::with_capacity(table.row_count());
    for index in 0..table.row_count() {
        tracks.push(AcbTrack {
            index,
            event_index: table.get_u16(index, "EventIndex").unwrap_or(u16::MAX),
        });
    }
    Ok(tracks)
}

fn read_track_events(utf: &UtfTable) -> Result<Vec<AcbTrackEvent>> {
    let Some(table) = open_first_table(utf, &["TrackEventTable", "CommandTable"])? else {
        return Ok(Vec::new());
    };

    let mut events = Vec::with_capacity(table.row_count());
    for index in 0..table.row_count() {
        events.push(AcbTrackEvent {
            index,
            references: table
                .get_data(index, "Command")
                .map(read_command_references)
                .unwrap_or_default(),
        });
    }
    Ok(events)
}

fn read_blocks(utf: &UtfTable) -> Result<Vec<AcbBlock>> {
    let Some(table) = open_optional_table(utf, "BlockTable")? else {
        return Ok(Vec::new());
    };

    let mut blocks = Vec::with_capacity(table.row_count());
    for index in 0..table.row_count() {
        let num_tracks = table.get_u16(index, "NumTracks").unwrap_or(0) as usize;
        blocks.push(AcbBlock {
            index,
            track_indices: table
                .get_data(index, "TrackIndex")
                .map(|data| read_i16_list(data, num_tracks))
                .unwrap_or_default(),
        });
    }
    Ok(blocks)
}

fn read_block_sequences(utf: &UtfTable) -> Result<Vec<AcbBlockSequence>> {
    let Some(table) = open_optional_table(utf, "BlockSequenceTable")? else {
        return Ok(Vec::new());
    };

    let mut sequences = Vec::with_capacity(table.row_count());
    for index in 0..table.row_count() {
        let num_tracks = table.get_u16(index, "NumTracks").unwrap_or(0) as usize;
        let num_blocks = table.get_u16(index, "NumBlocks").unwrap_or(0) as usize;
        sequences.push(AcbBlockSequence {
            index,
            track_indices: table
                .get_data(index, "TrackIndex")
                .map(|data| read_i16_list(data, num_tracks))
                .unwrap_or_default(),
            block_indices: table
                .get_data(index, "BlockIndex")
                .map(|data| read_i16_list(data, num_blocks))
                .unwrap_or_default(),
        });
    }
    Ok(sequences)
}

fn read_waveform_extensions(utf: &UtfTable) -> Result<Vec<AcbWaveformExtension>> {
    let Some(table) = open_optional_table(utf, "WaveformExtensionDataTable")? else {
        return Ok(Vec::new());
    };

    let mut extensions = Vec::with_capacity(table.row_count());
    for index in 0..table.row_count() {
        extensions.push(AcbWaveformExtension {
            index,
            loop_start: table.get_u32(index, "LoopStart").unwrap_or(0),
            loop_end: table.get_u32(index, "LoopEnd").unwrap_or(0),
        });
    }
    Ok(extensions)
}

fn read_references(data: &[u8]) -> Vec<AcbReference> {
    let mut references = Vec::with_capacity(data.len() / 4);
    for entry in data.as_chunks::<4>().0 {
        references.push(AcbReference {
            kind: u16::from_be_bytes([entry[0], entry[1]]),
            index: u16::from_be_bytes([entry[2], entry[3]]),
        });
    }
    references
}

fn read_command_references(data: &[u8]) -> Vec<AcbReference> {
    let mut references = Vec::new();
    let mut position = 0;

    while position + 3 <= data.len() {
        let code = u16::from_be_bytes([data[position], data[position + 1]]);
        let size = data[position + 2] as usize;
        position += 3;
        if position + size > data.len() {
            break;
        }

        if matches!(code, 2000 | 2003 | 2004) && size >= 4 {
            references.push(AcbReference {
                kind: u16::from_be_bytes([data[position], data[position + 1]]),
                index: u16::from_be_bytes([data[position + 2], data[position + 3]]),
            });
        }
        position += size;
    }

    references
}

fn read_i16_list(data: &[u8], count: usize) -> Vec<i16> {
    data.as_chunks::<2>()
        .0
        .iter()
        .take(count)
        .map(|entry| i16::from_be_bytes([entry[0], entry[1]]))
        .collect()
}

fn open_first_table(utf: &UtfTable, names: &[&str]) -> Result<Option<UtfTable>> {
    for name in names {
        if let Some(table) = open_optional_table(utf, name)? {
            return Ok(Some(table));
        }
    }
    Ok(None)
}

fn open_optional_table(utf: &UtfTable, name: &str) -> Result<Option<UtfTable>> {
    let Some((_, size)) = utf.get(0, name).and_then(|value| value.as_data()) else {
        return Ok(None);
    };
    if size == 0 {
        return Ok(None);
    }
    utf.open_subtable(0, name).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reference_items() {
        let references = read_references(&[0x00, 0x01, 0x00, 0x02]);
        assert_eq!(references, vec![AcbReference { kind: 1, index: 2 }]);
    }

    #[test]
    fn parses_note_on_command() {
        let references = read_command_references(&[0x07, 0xd0, 0x04, 0x00, 0x02, 0x00, 0x01]);
        assert_eq!(references, vec![AcbReference { kind: 2, index: 1 }]);
    }
}
