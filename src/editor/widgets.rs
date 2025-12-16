use nih_plug::prelude:: { Param };
use nih_plug_iced::{widgets::ParamMessage, *};
use std::sync::Arc;

pub struct ParamKnob<'a, P: Param> {
	state: &'a mut State,

    param: &'a P,

    height: Length,
    width: Length,
}

/// State for a [`ParamSlider`].
#[derive(Debug, Default)]
pub struct State {
	keyboard_modifiers: keyboard::Modifiers,
	/// Will be set to `true` if we're dragging the parameter. Resetting the parameter or entering a
	/// text value should not initiate a drag.
	drag_active: bool,
	/// Track clicks for double clicks.
	last_click: Option<mouse::Click>,
}

impl<'a, P: Param> ParamKnob<'a, P> {
	pub fn new(state: &'a mut State, param: &'a P) -> Self {
		Self {
			state,

			param,

			width: Length::Units(180),
			height: Length::Units(30),
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
}

impl<'a, P: Param> Widget<ParamMessage, backend::Renderer> for ParamKnob<'a, P> {
	fn width(&self) -> Length {
		self.width
	}

	fn height(&self) -> Length {
		self.height
	}

	fn layout(
			&self,
			renderer: &backend::Renderer,
			limits: &layout::Limits,
		) -> layout::Node {
		let limits = limits.width(self.width).height(self.height);
		let size = limits.resolve(Size::ZERO);

		layout::Node::new(size)
	}

	fn draw(
			&self,
			renderer: &mut backend::Renderer,
			style: &renderer::Style,
			layout: Layout<'_>,
			cursor_position: Point,
			viewport: &Rectangle,
		) {
		
	}

	fn on_event(
			&mut self,
			_event: Event,
			_layout: Layout<'_>,
			_cursor_position: Point,
			_renderer: &backend::Renderer,
			_clipboard: &mut dyn Clipboard,
			_shell: &mut Shell<'_, ParamMessage>,
		) -> event::Status {
		
	}
}
