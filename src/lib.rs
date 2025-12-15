use nih_plug::prelude::*;
use std::{default, sync::Arc};

fn ms_to_samples(ms: f32, sample_rate: f32) -> u32 {
	let samples = ms * sample_rate;

	return samples as u32;
}

struct Humanizer {
	params: Arc<HumanizerParams>,
}

#[derive(Params)]
struct HumanizerParams {
	/// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
	/// these IDs remain constant, you can rename and reorder these fields as you wish. The
	/// parameters are exposed to the host in the same order they were defined. In this case, this
	/// gain parameter is stored as linear gain while the values are displayed in decibels.
	#[id = "range"]
	pub range: FloatParam,
	#[id = "center"]
	pub center: FloatParam,
}

impl Default for Humanizer {
	fn default() -> Self {
		Self {
			params: Arc::new(HumanizerParams::default()),
		}
	}
}

impl Default for HumanizerParams {
	fn default() -> Self {
		Self {
			// This gain is stored as linear gain. NIH-plug comes with useful conversion functions
			// to treat these kinds of parameters as if we were dealing with decibels. Storing this
			// as decibels is easier to work with, but requires a conversion for every sample.
			center: FloatParam::new(
				"Center",
				0.0,
				FloatRange::Linear {
					min: - 0.5,
					max: 0.5,
				}
			),
			range: FloatParam::new(
				"Range",
				30.0,
				FloatRange::Linear {
					min: 0.0,
					max: 50.0,
				}
			)
		}
	}
}

impl Plugin for Humanizer {
	const NAME: &'static str = "Humanizer";
	const VENDOR: &'static str = "TheLazyCat00";
	const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
	const EMAIL: &'static str = "your@email.com";

	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	// The first audio IO layout is used as the default. The other layouts may be selected either
	// explicitly or automatically by the host or the user depending on the plugin API/backend.
	const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
		main_input_channels: NonZeroU32::new(2),
		main_output_channels: NonZeroU32::new(2),

		aux_input_ports: &[],
		aux_output_ports: &[],

		// Individual ports and the layout as a whole can be named here. By default these names
		// are generated as needed. This layout will be called 'Stereo', while a layout with
		// only one input and output channel would be called 'Mono'.
		names: PortNames::const_default(),
	}];


	const MIDI_INPUT: MidiConfig = MidiConfig::None;
	const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

	const SAMPLE_ACCURATE_AUTOMATION: bool = true;

	// If the plugin can send or receive SysEx messages, it can define a type to wrap around those
	// messages here. The type implements the `SysExMessage` trait, which allows conversion to and
	// from plain byte buffers.
	type SysExMessage = ();
	// More advanced plugins can use this to run expensive background tasks. See the field's
	// documentation for more information. `()` means that the plugin does not have any background
	// tasks.
	type BackgroundTask = ();

	fn params(&self) -> Arc<dyn Params> {
		self.params.clone()
	}

	fn initialize(
		&mut self,
		_audio_io_layout: &AudioIOLayout,
		_buffer_config: &BufferConfig,
		_context: &mut impl InitContext<Self>,
	) -> bool {
		let sample_rate = _buffer_config.sample_rate;
		
		let center = self.params.center.value();
		let range = self.params.range.value();

		let start = (center - 0.5) * range;

		_context.set_latency_samples(ms_to_samples(start, sample_rate));
		true
	}

	fn reset(&mut self) {
		// Reset buffers and envelopes here. This can be called from the audio thread and may not
		// allocate. You can remove this function if you do not need it.
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		_context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		let center = self.params.center.value();
		let range = self.params.range.value();
		let start = (center - 0.5) * range;
		let end = (center * 0.5) * range;
		
		ProcessStatus::Normal
	}
}

impl ClapPlugin for Humanizer {
	const CLAP_ID: &'static str = "com.your-domain.humanizer";
	const CLAP_DESCRIPTION: Option<&'static str> = Some("Humanizer FX");
	const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
	const CLAP_SUPPORT_URL: Option<&'static str> = None;

	// Don't forget to change these features
	const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Humanizer {
	const VST3_CLASS_ID: [u8; 16] = *b"Exactly16Chars!!";

	// And also don't forget to change these categories
	const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
		&[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(Humanizer);
nih_export_vst3!(Humanizer);
