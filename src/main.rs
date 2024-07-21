use std::{
    env, io::{stdout, Result}, process, sync::{
        Arc,
        Mutex,
        mpsc,
    }, thread
};

use ratatui::{
    backend::CrosstermBackend, crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    }, layout::Rect, style::Stylize, text::ToSpan, widgets::{Block, Borders, List, ListDirection, Paragraph, Wrap}, Terminal
};

// use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use twitch_irc::{
    login::StaticLoginCredentials, message::{PrivmsgMessage, ServerMessage}, ClientConfig, SecureTCPTransport, TwitchIRCClient
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

    // initialize send receive
    // let (tx, rx) = mpsc::channel();
    let (tx, rx) = mpsc::channel::<PrivmsgMessage>();


    // initialize terminal
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // initialize chats list

    // main terminal loop
    let channel_name_title = channel_name.clone();
    let terminal_handle = tokio::spawn(async move {
        let mut chats = Vec::new();
        loop {
            while let Ok(received) = rx.try_recv() {
                // chats.insert(0, received.channel_login + ":".to_string() + received.message_text);
                chats.insert(0, format!("{} : {}", received.sender.name, received.message_text));
            }
            terminal.draw(|frame| {
                let title = format!("{}{}{}",
                    "TwitchTerm - ",
                    channel_name_title,
                    " - type 'q' to quit");
                //let outer_border = Block::default().title("TwitchTerm - {} - Type 'q' to quit").borders(Borders::ALL);
                let outer_border = Block::default().title(title).borders(Borders::ALL);
                let outer_area = frame.size();

                // let chats_clone = chats.clone();
                
                // chats.push("test".to_string());
                let msg_list = List::new(chats.clone())
                    .direction(ListDirection::BottomToTop);
                let msg_list_area = Rect::new(1, 1, frame.size().width - 2, frame.size().height - 2);

                frame.render_widget(outer_border, outer_area);
                // frame.render_widget(inner_text, inner_text_area);

                frame.render_widget(msg_list, msg_list_area);


            }).unwrap();

            if event::poll(std::time::Duration::from_millis(16)).unwrap() {
                if let event::Event::Key(key) = event::read().unwrap() {
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

    let twitch_handler = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            match message {
                 ServerMessage::Privmsg(msg) => {
                    //tx.send(msg.message_text);
                    tx.send(msg);
                 },
                 _ => {}
            }
        }
        /*
        for _ in 0..5 {
            tx.send("sent".to_string());
            sleep(Duration::from_secs(1)).await;
        }
*/
    });

    client.join(channel_name.to_owned()).unwrap();


    terminal_handle.await.unwrap();
    // twitch_handler.await.unwrap();

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
