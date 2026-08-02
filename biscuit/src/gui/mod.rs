use biscuit::{GlobalFunction, Instructions, Machine, MachineOutput, bytecode::Command};
use ratatui::{
    Frame, crossterm::event::KeyCode, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style}, widgets::{Block, Borders, Paragraph},
};
use rustc_hash::FxHashMap;

/// App state. `selected` is the free index highlighted by the red box in the left panel.
pub struct App {
    code: String,
    machine: Machine,
    ip_map: FxHashMap<usize, usize>,
    err_state: bool,
}

impl App {
    pub fn new(instructions: Instructions, code: String) -> Self {
        let machine = Machine::new(instructions.clone(), 1);
        
        // Get ip map
        let mut ip = 0;
        let mut line_index = 0;
        let mut ip_map = FxHashMap::default();
        while ip < instructions.instructions.len() {
            ip_map.insert(ip, line_index);
            ip += match Command::try_from(instructions.instructions[ip]).unwrap() {
                Command::Call => 2,
                Command::Jmp | Command::Jnz | Command::Push => 9,
                _ => 1
            };
            line_index += 1;
        }
        Self {
            code: code,
            machine,
            ip_map,
            err_state: false,
        }
    }
}

impl App {
    pub fn draw(&self, f: &mut Frame) {
        // Split into two vertical panels.
        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(f.area());

        self.render_left(f, panels[0]);
        self.render_right(f, panels[1]);
    }

    /// Called on every arrow press. Fill this in yourself.
    pub fn on_key(&mut self, key_code: KeyCode) {
        if self.err_state {return;}
        match key_code {
            KeyCode::Char(' ') => {
                let copy = self.machine.clone();
                let result = match self.machine.run_to_call() {
                    Ok(MachineOutput::Call{func, args}) => {
                        match func {
                            GlobalFunction::Dbg => println!("Debug, {:?}", args),
                            GlobalFunction::Tick => println!("Tick, {:?}", args),
                            GlobalFunction::Interrupt => println!("Interrupt, {:?}", args),
                        };
                        Ok(())
                    },
                    Ok(MachineOutput::None) => Ok(()),
                    Err(e) => Err(e),
                };
                if let Err(_e) = result {
                    self.machine = copy;
                    self.err_state = true;
                };
            },
            _ => ()
        };
    }

    /// Left panel: one 3-row slot per line; the selected slot gets a red box.
    fn render_left(&self, f: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title("Code");
        let inner = block.inner(area);
        f.render_widget(block, area);

        let color = if self.err_state { Color::DarkGray } else { Color::White };

        let selected = self.ip_map[&self.machine.ip];

        for (i, line) in self.code.split('\n').enumerate() {
            if i as u16 >= inner.height {
                break;
            }
            let row = Rect {
                x: inner.x,
                y: inner.y + i as u16,
                width: inner.width,
                height: 1,
            };
            let mut para = Paragraph::new(line);
            let mut style = Style::default().fg(color);
            if i == selected {
                style = style.bg(Color::Red);
            }
            para = para.style(style);
            f.render_widget(para, row);
        }
    }

    /// Right panel: plain text, no box.
    fn render_right(&self, f: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title("Right");
        let inner = block.inner(area);
        f.render_widget(block, area);

        let color = if self.err_state { Color::DarkGray } else { Color::White };

        let text = self.machine.stack.iter().rev().map(|f| format!("{}", f)).collect::<Vec<_>>().join("\n");
        let mut para = Paragraph::new(text);
        let style = Style::default().fg(color);
        para = para.style(style);
        f.render_widget(para, inner);
    }

}