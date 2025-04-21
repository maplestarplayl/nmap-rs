use crate::app::{App, FocusZone};
use ratatui::{
    layout::{Constraint, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, text::Line, widgets::{BarChart, Block, Borders, Clear, Gauge, List, ListState, Paragraph, Widget, Wrap}, Frame
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    
    draw_first_tab(frame, app, frame.area());
    if app.input_trigger {
        popup(frame, app);
    }
}

fn draw_first_tab(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        // Constraint::Min(8),
        Constraint::Length(10),
        Constraint::Length(5)
    ])
    .split(area);

    let chunk2 = Layout::horizontal([
        Constraint::Percentage(30),
        Constraint::Percentage(70),
    ])
    .split(chunks[1]);
    draw_gauges(frame, app, chunks[0]);
    draw_charts(frame, app, chunk2[0]);
    draw_lists(frame, app, chunk2[1]);
    draw_input_box(frame, app, chunks[2]);
}

fn draw_gauges(frame: &mut Frame, app: &mut App, area: Rect) {
    // let block = Block::bordered().title("Graphs");
    // frame.render_widget(block, area);

    // let rand = rand::random::<f64>();
    let label = format!("{:.2}%", app.progress * 100.0);
    let gauge = Gauge::default()
        .block(Block::bordered().title("Gauge:"))
        .gauge_style(
            Style::default()
                .fg(Color::Magenta)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD),
        )
        // .use_unicode(app.enhanced_graphics)
        .label(label)
        .ratio(app.progress.into());
    frame.render_widget(gauge, area);
}

fn draw_charts(frame: &mut Frame, _app: &mut App, area: Rect) {
    let barchart = BarChart::default()
        .block(Block::bordered().title("Bar chart"))
        .data(&[
            ("memory", get_memory_usage() as u64),
            ("cpu", get_cpu_usage() as u64 * 100),
        ])
        .bar_width(8)
        .bar_gap(2)
        // .bar_set(if app.enhanced_graphics {
        //     symbols::bar::NINE_LEVELS
        // } else {
        //     symbols::bar::THREE_LEVELS
        // })
        .value_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::ITALIC),
        )
        .label_style(Style::default().fg(Color::Yellow))
        .bar_style(Style::default().fg(Color::Green));
    frame.render_widget(barchart, area);
}

fn draw_lists(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(area);

    // list1: address list
    let list1 = List::new(app.targets.clone())
        .block(Block::bordered().title("Address List").title_bottom(app.targets.len().to_string()))
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC),
        );
    
    let target = app.targets.get(app.targets_selected);
    let data2 = match target {
        Some(target) => match app.port_results.get(target) {
            Some(results) => results.clone(),
            None => Vec::new(),
        },
        None => Vec::new(),
    };
    let list2 = List::new(data2)
        .block(Block::bordered().title("Port List"))
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC),
        );

    let mut state1 = ListState::default();
    if !app.targets.is_empty() && app.focus_zone == FocusZone::AddressList {
        state1.select(Some(app.targets_selected));
    }
    let mut state2 = ListState::default();
    if !app.port_results.is_empty() && app.focus_zone == FocusZone::PortList {
        state2.select(Some(app.port_results_selected));
    }   
    
    frame.render_stateful_widget(list1, chunks[0], &mut state1);
    frame.render_stateful_widget(list2, chunks[1], &mut state2);
}

fn draw_input_box(frame: &mut Frame, app: &mut App, area: Rect) {
    let instructions = Line::from(vec![
        " Change Focus ".into(),
        "<Tab>".blue().bold(),
        " Input ".into(),
        "<Enter>".blue().bold(),
        " Execute ".into(),
        "<E>".blue().bold(),
        " Quit ".into(),
        "<Q> ".blue().bold(),
    ]);
    let input_list = List::new(["Address", "Port"])
        .block(Block::bordered().title("Input").title_bottom(instructions.centered()))
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC),
        );

    let mut state = ListState::default();
    if app.focus_zone == FocusZone::InputList {
        state.select(Some(app.input_selected));
    }
    frame.render_stateful_widget(input_list, area, &mut state);
}

fn popup(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let popup_area = Rect {
        x: area.width / 4,
        y: area.height / 3,
        width: area.width / 2,
        height: area.height / 3,
    };
    let text = match app.input_selected {
        0 => app.target_input.clone(),
        1 => app.port_input.clone(),
        _ => String::from("dsadsad"),
    };
    let bad_popup = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .style(Style::new().yellow().bg(Color::Black))
        .block(
            Block::new()
                .title("Input Box")
                .title_style(Style::new().white().bold())
                .borders(Borders::ALL)
                .border_style(Style::new().red()),
        );
    frame.render_widget(bad_popup, popup_area); 
}

fn get_memory_usage() -> f64 {
    let mut sys = sysinfo::System::new();
    sys.refresh_all();

    let pid = sysinfo::get_current_pid().expect("Failed to get current process ID");
    if let Some(process) = sys.process(pid) {
        let memory_kb = process.memory(); // Memory in kilobytes
        return memory_kb as f64 / 1024.0 / 1024.0;
    }
    0.0
}

fn get_cpu_usage() -> f64 {
    let mut sys = sysinfo::System::new();
    sys.refresh_all();

    let pid = sysinfo::get_current_pid().expect("Failed to get current process ID");
    if let Some(process) = sys.process(pid) {
        let cpu_usage = process.cpu_usage(); // CPU usage in percentage
        return cpu_usage as f64;
    }
    0.0
}


/// Helper function to create a centered rect using a percentage of the available rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Calculate the width and height based on percentages
    let popup_width = r.width * percent_x / 100;
    let popup_height = r.height * percent_y / 100;
    
    // Create a centered rect using Layout
    let popup_rect = Rect::new(
        // Center horizontally
        r.x + (r.width - popup_width) / 2,
        // Center vertically
        r.y + (r.height - popup_height) / 2,
        // Set width and height
        popup_width,
        popup_height,
    );
    
    popup_rect
}