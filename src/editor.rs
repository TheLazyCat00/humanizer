use nih_plug::prelude::{ Editor, GuiContext};
use nih_plug_iced::widgets as nih_widgets;
use nih_plug_iced::*;
use std::sync::Arc;

mod widgets;
use widgets::*;
use crate::HumanizerParams;

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<IcedState> {
	IcedState::from_size(200, 150)
}

pub(crate) fn create(
	params: Arc<HumanizerParams>,
	editor_state: Arc<IcedState>,
) -> Option<Box<dyn Editor>> {
	create_iced_editor::<HumanizerEditor>(editor_state, params)
}

struct HumanizerEditor {
	params: Arc<HumanizerParams>,
	context: Arc<dyn GuiContext>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
	ParamUpdate(nih_widgets::ParamMessage),
}

impl IcedEditor for HumanizerEditor {
	type Executor = executor::Default;
	type Message = Message;
	type InitializationFlags = Arc<HumanizerParams>;

	fn new(
		params: Self::InitializationFlags,
		context: Arc<dyn GuiContext>,
	) -> (Self, Command<Self::Message>) {
		let editor = HumanizerEditor {
			params,
			context,
		};

		(editor, Command::none())
	}

	fn context(&self) -> &dyn GuiContext {
		self.context.as_ref()
	}

	fn update(
		&mut self,
		_window: &mut WindowQueue,
		message: Self::Message,
	) -> Command<Self::Message> {
		match message {
			Message::ParamUpdate(message) => self.handle_param_message(message),
		}

		Command::none()
	}

	fn view(&mut self) -> Element<'_, Self::Message> {
		let knob = ParamKnob::new(&self.params.range);
		Column::new()
			.align_items(Alignment::Center)
			.push(
				Text::new("Gain GUI")
					.font(assets::NOTO_SANS_LIGHT)
					.size(40)
					.height(50.into())
					.width(Length::Fill)
					.horizontal_alignment(alignment::Horizontal::Center)
					.vertical_alignment(alignment::Vertical::Bottom),
			)
			.push(
				Text::new("Gain")
					.height(20.into())
					.width(Length::Fill)
					.horizontal_alignment(alignment::Horizontal::Center)
					.vertical_alignment(alignment::Vertical::Center),
			)
			.push(knob.map(Message::ParamUpdate))
			.into()
	}

	fn background_color(&self) -> nih_plug_iced::Color {
		nih_plug_iced::Color {
			r: 0.98,
			g: 0.98,
			b: 0.98,
			a: 1.0,
		}
	}
}
