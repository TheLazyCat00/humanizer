use nih_plug::prelude::*;
use std:: { sync::Arc };
use noise:: { Perlin, NoiseFn };

fn ms_to_samples(ms: f32, sample_rate: f32) -> u32 {
	let samples = ms * sample_rate;

	return samples as u32;
}

struct Humanizer {
	params: Arc<HumanizerParams>,
	delay_line: Vec<[f32; 2]>,
	write_idx: usize,
	max_delay_samples: usize,
	perlin: Perlin,
	noise_pos: f64,
	frequency: f64,
	sample_rate: f32,
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
		self.sample_rate = _buffer_config.sample_rate;
		
		let center = self.params.center.value();
		let range = self.params.range.value();

		let start = (center - 0.5) * range;

		_context.set_latency_samples(ms_to_samples(start, self.sample_rate));
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
		let sample_rate = self.sample_rate as f64;
		let num_samples = buffer.samples();

		let center = self.params.center.smoothed.next();
		let range = self.params.range.smoothed.next();
		let start = (center - 0.5) * range;
		let end = (center * 0.5) * range;

		let noise_value = self.perlin.get([self.noise_pos]);
		let output_sample = noise_value as f32;
		self.noise_pos += self.frequency * (1.0 / sample_rate);
		let delay_time_samples =
		// NIH-plug's `iter_samples()` allows you to iterate over all channels/samples simultaneously
		for (sample_idx, mut sample_data) in buffer.iter_samples().enumerate() {
		}

		// Update the write index for the next block
		self.write_idx = (self.write_idx + num_samples) % self.max_delay_samples;

		ProcessStatus::Normal
	}
fn process(
        &mut self,
        buffer: &mut Buffer<'_>,
        _aux: &mut AuxiliaryBuffers<'_>,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // We use the sample rate from the buffer for accuracy
        let sample_rate = buffer.sample_rate() as f64;
        let num_samples = buffer.samples();

        // --- 1. Get Smoothed Modulation Parameters ---
        // 'center' controls the base delay offset (e.g., 0.5 for half the range)
        let center = self.params.center.smoothed.next(); 
        // 'range' controls the total span of the shift (e.g., max 10ms shift)
        let range = self.params.range.smoothed.next(); 
        // 'frequency' controls the speed of the LFO (Perlin noise)
        let frequency = self.params.frequency.smoothed.next() as f64; 
        
        // Calculate the modulation limits in *samples*
        // The noise shifts the delay time around the center point.
        // We use a small maximum delay (e.g., 2048 samples) just to be safe, 
        // but the actual shifting range is determined by 'range'.
        let max_shift_samples = 2048.0; 
        
        // Base delay time in samples (offset from current sample)
        let base_delay = center * range * max_shift_samples; 
        // Modulation depth in samples (total shift range)
        let modulation_depth = range * max_shift_samples; 

        // Calculate the change in noise position per sample
        let noise_step = frequency / sample_rate;
        
        // --- 2. Process Audio Samples ---
        for (sample_idx, mut sample_data) in buffer.iter_samples().enumerate() {
            let sample_idx_f64 = sample_idx as f64;
            
            // a) Calculate the instantaneous, smooth noise-modulated delay time
            let current_noise_pos = self.noise_pos + sample_idx_f64 * noise_step;
            let current_noise_value = self.perlin.get([current_noise_pos]); // noise is in [-1.0, 1.0]

            // Modulate the delay time: Base Delay + (Noise Value * Modulation Depth)
            // Note: This is the total delay *from the input*, not just the LFO amount.
            let delay_time_samples_f32 = (base_delay + current_noise_value as f32 * modulation_depth)
                .max(0.0) // Ensure delay time is non-negative
                .min(self.max_delay_samples as f32 - 1.0); // Stay within buffer bounds

            // b) Separate the integer and fractional parts for interpolation
            let delay_time_samples = delay_time_samples_f32;
            let read_delay_idx_i = delay_time_samples.floor() as usize; // Integer part
            let fraction = delay_time_samples.fract(); // Fractional part

            // c) Calculate the circular buffer read indices
            // P1 is the sample *before* the fractional read point
            // P2 is the sample *after* the fractional read point
            let current_write_idx = (self.write_idx + sample_idx) % self.max_delay_samples;
            
            // Note: The read index counts *back* from the current write index
            let p1_read_idx = (current_write_idx + self.max_delay_samples - read_delay_idx_i) 
                              % self.max_delay_samples;
            let p2_read_idx = (p1_read_idx + self.max_delay_samples - 1) 
                              % self.max_delay_samples;

            // d) Process each channel
            for channel_idx in 0..buffer.channels() {
                let input_sample = sample_data[channel_idx];
                
                // Read the two required samples
                let p1 = self.delay_line[p1_read_idx][channel_idx];
                let p2 = self.delay_line[p2_read_idx][channel_idx];
                
                // --- LINEAR INTERPOLATION --- 
                // delayed_sample = P1 + (P2 - P1) * fraction
                let delayed_sample = p1 + (p2 - p1) * fraction;

                // --- Calculate Output and Write Back ---
                // For a simple shift effect, we typically just output the wet signal
                let wet_output = delayed_sample;
                
                // For a flanger/chorus, you might mix:
                // let output = input_sample + wet_output; 
                
                // For a simple shift, we just output the delayed audio (wet)
                let output = wet_output; 

                // Write the input sample to the circular buffer for later reading
                self.delay_line[current_write_idx][channel_idx] = input_sample;

                // Update the output buffer
                sample_data[channel_idx] = output;
            }
        }
        
        // --- 3. Advance State for Next Block ---
        // Advance the noise position for the start of the next block
        self.noise_pos += num_samples as f64 * noise_step;
        
        // Update the write index for the next block
        self.write_idx = (self.write_idx + num_samples) % self.max_delay_samples;

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
