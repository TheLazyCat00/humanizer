use nih_plug::prelude::*;
use std::sync::Arc;
use std::num::NonZeroU32;
use noise::{Perlin, NoiseFn};

struct Humanizer {
	params: Arc<HumanizerParams>,
	delay_line: Vec<[f32; 2]>,
	write_idx: usize,
	max_delay_samples: usize,
	perlin: Perlin,
	noise_pos: f64,
	sample_rate: f32,
}

#[derive(Params)]
struct HumanizerParams {
	#[id = "range"]
	pub range: FloatParam,
	#[id = "center"]
	pub center: FloatParam,
	#[id = "frequency"]
	pub frequency: FloatParam,
}

impl Humanizer {
	fn ms_to_samples(&self, ms: f32) -> u32 {
		let samples = ms * self.sample_rate / 1000.0;
		samples as u32
	}
}
// --- Default Implementations ---
impl Default for Humanizer {
	fn default() -> Self {
		Self {
			params: Arc::new(HumanizerParams::default()),
			delay_line: Vec::new(),
			write_idx: 0,
			max_delay_samples: 0, 
			perlin: Perlin::new(42), 
			noise_pos: 0.0,
			sample_rate: 0.0, 
		}
	}
}

impl Default for HumanizerParams {
	fn default() -> Self {
		Self {
			// ... center and range stay the same ...
			center: FloatParam::new(
				"Center", 0.0,
				FloatRange::Linear { min: -0.5, max: 0.5 }
			).with_unit("Normalized"),

			range: FloatParam::new(
				"Range", 10.0,
				FloatRange::Linear { min: 0.0, max: 50.0 }
			).with_unit(" ms"),

			// CHANGE: Update label and range. 
			// 0.25 = 1 cycle every 4 beats. 4.0 = 4 cycles per beat.
			frequency: FloatParam::new(
				"Speed", // Renamed for clarity
				1.0, 
				FloatRange::Linear { min: 0.1, max: 8.0 } 
			).with_unit(" Cycles/Beat"),
		}
	}
}

// --- Plugin Implementation ---
impl Plugin for Humanizer {
	const NAME: &'static str = "Humanizer";
	const VENDOR: &'static str = "TheLazyCat00";
	const URL: &'static str = "https://youtu.be/dQw4w9WgXcQ"; // Placeholder
	const EMAIL: &'static str = "your@email.com";
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
		main_input_channels: NonZeroU32::new(2),
		main_output_channels: NonZeroU32::new(2),
		aux_input_ports: &[],
		aux_output_ports: &[],
		names: PortNames::const_default(),
	}];

	const MIDI_INPUT: MidiConfig = MidiConfig::None;
	const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
	const SAMPLE_ACCURATE_AUTOMATION: bool = true;
	type SysExMessage = ();
	type BackgroundTask = ();

	fn params(&self) -> Arc<dyn Params> {
		self.params.clone()
	}

	fn initialize(
		&mut self,
		_audio_io_layout: &AudioIOLayout,
		buffer_config: &BufferConfig,
		_context: &mut impl InitContext<Self>,
	) -> bool {
		self.sample_rate = buffer_config.sample_rate;

		const MAX_DELAY_MS: f32 = 60.0;
		let max_samples = self.ms_to_samples(MAX_DELAY_MS) as usize;

		self.delay_line.resize(max_samples, [0.0; 2]);
		self.max_delay_samples = max_samples;

		// Report latency
		const REQUIRED_LATENCY_MS: f32 = 50.0;
		_context.set_latency_samples(self.ms_to_samples(REQUIRED_LATENCY_MS));

		true
	}

	fn reset(&mut self) {
		// Clear buffers on reset
		for sample in self.delay_line.iter_mut() {
			*sample = [0.0; 2];
		}
		self.write_idx = 0;
	}

	fn process(
		&mut self,
		buffer: &mut Buffer<'_>,
		_aux: &mut AuxiliaryBuffers<'_>,
		_context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		let sample_rate = self.sample_rate as f64;
		let num_samples = buffer.samples();

		// --- 1. Get Tempo from Host ---
		let transport = _context.transport();
		// Default to 120.0 BPM if the host doesn't provide it or isn't playing
		let tempo = transport.tempo.unwrap_or(120.0);

		let range_ms = self.params.range.smoothed.next(); 
		let center = self.params.center.smoothed.next(); 

		// This parameter is now "Cycles per Beat"
		let cycles_per_beat = self.params.frequency.smoothed.next() as f64;

		// --- 2. Convert Beats to Hz ---
		// Formula: Hz = (BPM / 60) * CyclesPerBeat
		// Example: 120 BPM = 2 beats/sec. If param is 1.0, freq is 2 Hz.
		let frequency_hz = (tempo / 60.0) * cycles_per_beat;

		let max_range_samples = self.ms_to_samples(range_ms) as f32;
		let center_norm = center + 0.5; 
		let base_delay = max_range_samples * (1.0 - center_norm); 
		let modulation_depth = max_range_samples * 0.5; 

		// Use the calculated Hz for the step size
		let noise_step = frequency_hz / sample_rate;

		for (sample_idx, samples) in buffer.iter_samples().enumerate() {
			let sample_idx_f64 = sample_idx as f64;

			let current_noise_pos = self.noise_pos + sample_idx_f64 * noise_step;
			let current_noise_value = self.perlin.get([current_noise_pos]) as f32; 

			let delay_time_samples_f32 = (base_delay + current_noise_value * modulation_depth)
				.max(0.0)
				.min(self.max_delay_samples as f32 - 2.0); 

			let read_delay_idx_i = delay_time_samples_f32.floor() as usize; 
			let fraction = delay_time_samples_f32.fract(); 

			let current_write_idx = (self.write_idx + sample_idx) % self.max_delay_samples;

			let p1_read_idx = (current_write_idx + self.max_delay_samples - read_delay_idx_i) 
			% self.max_delay_samples;
			let p2_read_idx = (p1_read_idx + self.max_delay_samples - 1) 
			% self.max_delay_samples;

			for (channel_idx, sample) in samples.into_iter().enumerate() {
				let input_sample = *sample;

				let p1 = self.delay_line[p1_read_idx][channel_idx];
				let p2 = self.delay_line[p2_read_idx][channel_idx];

				let delayed_sample = p1 + (p2 - p1) * fraction;

				self.delay_line[current_write_idx][channel_idx] = input_sample;

				*sample = delayed_sample;
			}
		}

		self.noise_pos += num_samples as f64 * noise_step;
		self.write_idx = (self.write_idx + num_samples) % self.max_delay_samples;

		ProcessStatus::Normal
	}
}

// --- Exports ---
impl ClapPlugin for Humanizer {
	const CLAP_ID: &'static str = "com.your-domain.humanizer";
	const CLAP_DESCRIPTION: Option<&'static str> = Some("Humanizer FX");
	const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
	const CLAP_SUPPORT_URL: Option<&'static str> = None;
	const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Humanizer {
	const VST3_CLASS_ID: [u8; 16] = *b"LazeyCatHumzer!!"; 
	const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
	&[Vst3SubCategory::Fx, Vst3SubCategory::Modulation];
}

nih_export_clap!(Humanizer);
nih_export_vst3!(Humanizer);
