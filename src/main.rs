use std::{
    io::{stdout, Result},
    env,
    process,
    thread, time::Duration, sync::mpsc,  // thread things
};

use ratatui::{
    backend::CrosstermBackend, crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    }, layout::Rect, style::Stylize, text::ToSpan, widgets::{Block, Borders, List, ListDirection, ListItem, Paragraph, Wrap}, Terminal
};

use tokio::sync::Mutex;
use twitch_irc::{
    login::StaticLoginCredentials,
    TwitchIRCClient,
    ClientConfig,
    SecureTCPTransport,
    message::ServerMessage,
};

#[tokio::main]
pub async fn main() -> Result<()> {
    // get channel name
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run [channel]");
        process::exit(1);

    }
    let channel_name = args[1].clone();
    println!("Now reading from {}...", channel_name);

    // initialize terminal
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // initialize send receive
    let (tx, rx) = mpsc::channel();

    let mut chats = Vec::new();
    let channel_name_term_clone = channel_name.clone();

    // main terminal loop
    let terminal_handler = thread::spawn(move || {
        loop {
            let received = rx.recv().unwrap();
            chats.push(received);
            if let Err(e) = terminal.draw(|frame| {
                let title = format!("{}{}{}",
                    "TwitchTerm - ",
                    channel_name_term_clone,
                    " - type 'q' to quit");
                //let outer_border = Block::default().title("TwitchTerm - {} - Type 'q' to quit").borders(Borders::ALL);
                let outer_border = Block::default().title(title).borders(Borders::ALL);
                let outer_area = frame.size();

                chats.push("test".to_string());
                let msg_list = List::new(chats.clone())
                    .direction(ListDirection::BottomToTop);
                let msg_list_area = Rect::new(1, 1, frame.size().width - 2, frame.size().height - 2);



                frame.render_widget(outer_border, outer_area);
                // frame.render_widget(inner_text, inner_text_area);

                frame.render_widget(msg_list, msg_list_area);


            }) {
                println!("Error: could not draw to terminal");
                process::exit(1);
            }

            if let Err(e) = event::poll(std::time::Duration::from_millis(16)) {
                println!("Error handling poll event");
                process::exit(1);
            } else {
                if let Ok(event::Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
                    }
                }

            }
        }

    });

    // join twitch chat
    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    // main twitch loop
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            // println!("Received message: {:?}", message);
            match message {
                ServerMessage::Privmsg(message) => {
                    tx.send(message.message_text);
                }

                _ => {}

            }
        }
    });


    client.join(channel_name.to_owned()).unwrap();
    terminal_handler.join().unwrap();

    join_handle.await.unwrap();
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
