use iced_audio::*;

use nih_plug_iced::*;
use nih_plug::prelude::Param;
use nih_plug_iced::backend::widget;
use nih_plug_iced::backend::Renderer;
use nih_plug_iced::text::Renderer as TextRenderer;
use nih_plug_iced::widgets::ParamMessage;

/// When shift+dragging a parameter, one pixel dragged corresponds to this much change in the
/// noramlized parameter.
const GRANULAR_DRAG_MULTIPLIER: f32 = 0.1;

/// The thickness of this widget's borders.
const BORDER_WIDTH: f32 = 1.0;

/// A slider that integrates with NIH-plug's [`Param`] types.
///
/// TODO: There are currently no styling options at all
/// TODO: Handle scrolling for steps (and shift+scroll for smaller steps?)
pub struct ParamKnob<'a, P>
where
	P: Param,
{
	param: &'a P,
	width: Length,
	height: Length,
	text_size: Option<u16>,
	font: Font,
}

/// An internal message for intercep- I mean handling output from the embedded [`TextInpu`] widget.
#[derive(Debug, Clone)]
enum TextInputMessage {
	/// A new value was entered in the text input dialog.
	Value(String),
	/// Enter was pressed.
	Submit,
}

/// The default text input style with the border removed.
struct TextInputStyle;

impl widget::text_input::StyleSheet for TextInputStyle {
	fn active(&self) -> widget::text_input::Style {
		widget::text_input::Style {
			background: Background::Color(Color::TRANSPARENT),
			border_radius: 0.0,
			border_width: 0.0,
			border_color: Color::TRANSPARENT,
		}
	}

	fn focused(&self) -> widget::text_input::Style {
		self.active()
	}

	fn placeholder_color(&self) -> Color {
		Color::from_rgb(0.7, 0.7, 0.7)
	}

	fn value_color(&self) -> Color {
		Color::from_rgb(0.3, 0.3, 0.3)
	}

	fn selection_color(&self) -> Color {
		Color::from_rgb(0.8, 0.8, 1.0)
	}
}

impl<'a, P> ParamKnob<'a, P>
where
	P: Param,
{
	/// Creates a new [`ParamSlider`] for the given parameter.
	pub fn new(param: &'a P) -> Self {
		Self {
			param,

			width: Length::Units(180),
			height: Length::Units(30),
			text_size: None,
			font: <Renderer as TextRenderer>::Font::default(),
		}
	}

	/// Sets the width of the [`ParamSlider`].
	pub fn width(mut self, width: Length) -> Self {
		self.width = width;
		self
	}

	/// Sets the height of the [`ParamSlider`].
	pub fn height(mut self, height: Length) -> Self {
		self.height = height;
		self
	}

	/// Sets the text size of the [`ParamSlider`].
	pub fn text_size(mut self, size: u16) -> Self {
		self.text_size = Some(size);
		self
	}

	/// Sets the font of the [`ParamSlider`].
	pub fn font(mut self, font: Font) -> Self {
		self.font = font;
		self
	}
}

impl<'a, P: Param> Widget<ParamMessage, Renderer> for ParamKnob<'a, P> {
	fn width(&self) -> Length {
		self.width
	}

	fn height(&self) -> Length {
		self.height
	}

	fn layout(&self, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
		let limits = limits.width(self.width).height(self.height);
		let size = limits.resolve(Size::ZERO);

		layout::Node::new(size)
	}

	fn on_event(
			&mut self,
			_event: Event,
			_layout: Layout<'_>,
			_cursor_position: Point,
			_renderer: &Renderer,
			_clipboard: &mut dyn Clipboard,
			_shell: &mut Shell<'_, ParamMessage>,
		) -> event::Status {

	}

	fn mouse_interaction(
			&self,
			_layout: Layout<'_>,
			_cursor_position: Point,
			_viewport: &Rectangle,
			_renderer: &Renderer,
		) -> mouse::Interaction {
		
	}

	fn draw(
			&self,
			renderer: &mut Renderer,
			style: &renderer::Style,
			layout: Layout<'_>,
			cursor_position: Point,
			viewport: &Rectangle,
		) {
		
		let param_ptr = self.param.as_ptr();

		
		let knob = Knob::new(
			NormalParam {
				value: Normal::from_clipped(self.param.default_normalized_value()),
				default: Normal::from_clipped(self.param.default_normalized_value()),
			},
			move |normal| {
				ParamMessage::SetParameterNormalized(param_ptr, normal.as_f32())
			}
		).style(style);

		let hi = 1;
	}
}

impl<'a, P: Param> ParamKnob<'a, P> {
	/// Convert this [`ParamSlider`] into an [`Element`] with the correct message. You should have a
	/// variant on your own message type that wraps around [`ParamMessage`] so you can forward those
	/// messages to
	/// [`IcedEditor::handle_param_message()`][crate::IcedEditor::handle_param_message()].
	pub fn map<Message, F>(self, f: F) -> Element<'a, Message>
	where
		Message: 'static,
		F: Fn(ParamMessage) -> Message + 'static,
	{
		Element::from(self).map(f)
	}
}

impl<'a, P: Param> From<ParamKnob<'a, P>> for Element<'a, ParamMessage> {
	fn from(widget: ParamKnob<'a, P>) -> Self {
		Element::new(widget)
	}
}
