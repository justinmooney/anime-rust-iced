use std::sync::Arc;

use iced::widget::{
    button, column, progress_bar, row, scrollable, text, text_input, Column, Space,
};
use iced::{alignment, executor, window, Application, Command, Element, Length, Settings, Theme};

use anime::{get_downloader, load_data, AnimeItem, AnimeItemList};

fn main() -> Result<(), iced::Error> {
    AnimeApp::run(Settings {
        window: window::Settings {
            size: iced::Size {
                width: 1200.0,
                height: 800.0,
            },
            resizable: (true),
            decorations: (true),
            ..Default::default()
        },
        ..Default::default()
    })
}

struct AnimeApp {
    data: Arc<AnimeItemList>,
    display_content: AnimeItem,
    load_progress: f32,
}

#[derive(Clone, Debug)]
enum Message {
    DataLoaded(Arc<AnimeItemList>),
    ButtonPressed(AnimeItem),
}

impl Application for AnimeApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                data: Arc::new(AnimeItemList::default()),
                display_content: AnimeItem::default(),
                load_progress: 0.0,
            },
            Command::perform(get_animes(), Message::DataLoaded),
        )
    }

    fn title(&self) -> String {
        String::from("Animes")
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::DataLoaded(animes) => {
                self.data = animes;
                Command::none()
            }
            Message::ButtonPressed(content) => {
                self.display_content = content;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut titles = Column::new()
            .width(620)
            .align_items(alignment::Alignment::Center);

        let search_box = text_input("search...", "").padding(5);

        for a in self.data.items.iter() {
            let title_widget = button(text(a.title.clone()))
                .padding(10)
                .width(Length::FillPortion(1))
                .on_press(Message::ButtonPressed(a.clone()));
            titles = titles.push(title_widget);
            titles = titles.push(Space::with_height(3))
        }

        let anime_list = scrollable(titles)
            .height(Length::FillPortion(1))
            .width(Length::FillPortion(1));

        let list_widget = column![search_box, anime_list];

        let a = &self.display_content;

        let display_widget = column![
            text(&a.title)
                .size(32)
                .width(Length::FillPortion(1))
                .horizontal_alignment(alignment::Horizontal::Center),
            text(a.display_date())
                .width(Length::FillPortion(1))
                .horizontal_alignment(alignment::Horizontal::Center),
            Space::with_height(24),
            text(&a.synopsis)
                .width(Length::FillPortion(1))
                .height(Length::FillPortion(1)),
        ]
        .width(600)
        .padding(10);

        let mut layout = column![];
        if (1.0..99.0).contains(&self.load_progress) {
            let progress = progress_bar(0.0..=100.0, self.load_progress);
            layout = layout.push(progress);
        }
        layout = layout.push(row![list_widget, display_widget]);

        layout.into()
    }
}

async fn get_animes() -> Arc<AnimeItemList> {
    let mut downloader = get_downloader().unwrap();
    while downloader.has_remaining() {
        println!("{:#?}", downloader);
        downloader.fetch_next();
    }

    Arc::new(load_data().unwrap())
}
