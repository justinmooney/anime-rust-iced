use std::sync::Arc;

use iced::widget::{button, container, row, scrollable, text, Column};
use iced::{executor, Application, Command, Element, Length, Settings, Theme};

use anime::{load_data, Anime};

fn main() -> Result<(), iced::Error> {
    AnimeApp::run(Settings::default())
}

struct AnimeApp {
    data: Arc<Vec<Anime>>,
    display_content: Anime,
}

#[derive(Clone, Debug)]
enum Message {
    DataLoaded(Arc<Vec<Anime>>),
    ButtonPressed(Anime),
}

impl Application for AnimeApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                data: Arc::new(Vec::new()),
                display_content: Anime::new(),
            },
            Command::perform(get_animes(), Message::DataLoaded),
        )
    }

    fn title(&self) -> String {
        String::from("Animes")
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::DataLoaded(animes) => {
                self.data = animes;
                Command::none()
            }
            Message::ButtonPressed(content) => {
                println!("FUUUUCK");
                self.display_content = content;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        //text_editor(&self.content).into()
        let mut titles = Column::new();

        for a in self.data.iter() {
            let title_widget = button(text(a.title.clone()))
                .padding(10)
                .width(400)
                .on_press(Message::ButtonPressed(a.clone()));
            titles = titles.push(title_widget);
        }

        let anime_list = scrollable(titles).height(Length::FillPortion(1)).width(400);

        let list_widget = container(anime_list);
        let display_content = format!(
            "{}\n{} - {}\n\n{}",
            &self.display_content.title,
            &self.display_content.start_date,
            &self.display_content.end_date,
            &self.display_content.synopsis
        );
        let display_widget = text(display_content);

        row![list_widget, display_widget].into()
    }
}

async fn get_animes() -> Arc<Vec<Anime>> {
    Arc::new(load_data().unwrap())
}
